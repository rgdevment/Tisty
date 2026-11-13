import { syntaxTree } from "@codemirror/language";
import {
  EditorSelection,
  EditorState,
  type Range,
  StateEffect,
  StateField,
  type Transaction,
} from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  type KeyBinding,
  WidgetType,
} from "@codemirror/view";
import type { Tree, TreeCursor } from "@lezer/common";
import { frontLines } from "../front";
import { cellEdited, cellPressed, rowAdded } from "./celling";
import { hosted } from "./hosted";
import { fencePairs, marksShown } from "./seeing";
import { columnAsked, columnWidened, rowAsked } from "./tableMenus";

type Rich = { readonly from: number; readonly to: number };

const LINE_PX = 26;
const FRONT_AT_MOST = 8 * 1024;

const pictured = (state: EditorState, cursor: TreeCursor): boolean => {
  const node = cursor.node;
  let pictures = 0;
  for (let child = node.firstChild; child; child = child.nextSibling) {
    if (child.name !== "Image") return false;
    pictures++;
  }
  const rest = state.sliceDoc(node.from, node.to).replace(/!\[[^\]]*\]\([^)]*\)/g, "");
  return pictures > 0 && !rest.trim();
};

const IDEA = /^\s*>\s*\[!idea\]/i;

const isRich = (state: EditorState, cursor: TreeCursor): boolean => {
  if (cursor.name === "Table" || cursor.name === "HTMLBlock" || cursor.name === "FencedCode")
    return true;
  if (cursor.name === "Blockquote") return IDEA.test(state.doc.lineAt(cursor.from).text);
  return cursor.name === "Paragraph" && pictured(state, cursor);
};

const foldsIn = (state: EditorState): Rich[] =>
  fencePairs(state)
    .filter(({ open }) => state.doc.sliceString(open.from, open.to).startsWith(":::details"))
    .map(({ open, shut }) => ({ from: state.doc.lineAt(open.from).from, to: shut.to }));

const richKnown = new WeakMap<Tree, Rich[]>();

const richIn = (state: EditorState): Rich[] => {
  const tree = syntaxTree(state);
  const kept = richKnown.get(tree);
  if (kept) return kept;
  const found: Rich[] = [];
  richKnown.set(tree, found);
  const folds = foldsIn(state);
  const folded = (at: number) => folds.some((one) => one.from <= at && at <= one.to);
  const cursor = tree.topNode.cursor();
  if (cursor.firstChild()) {
    do {
      if (folded(cursor.from) || !isRich(state, cursor)) continue;
      const from = state.doc.lineAt(cursor.from).from;
      const end =
        cursor.to > 0 && state.doc.sliceString(cursor.to - 1, cursor.to) === "\n"
          ? cursor.to - 1
          : cursor.to;
      found.push({ from, to: state.doc.lineAt(end).to });
    } while (cursor.nextSibling());
  }
  found.push(...folds);
  found.sort((a, b) => a.from - b.from);
  return found;
};

const reveal = StateEffect.define<number | null>();

const revealed = StateField.define<number | null>({
  create: () => null,
  update: (was, tr) => {
    for (const one of tr.effects) if (one.is(reveal)) return one.value;
    if (was === null) return null;
    const at = tr.changes.mapPos(was);
    const line = tr.state.doc.lineAt(at);
    const block = richIn(tr.state).find((one) => one.from <= line.from && line.from <= one.to);
    const head = tr.state.selection.main.head;
    return block && head >= block.from && head <= block.to ? at : null;
  },
});

class Drawn extends WidgetType {
  constructor(
    readonly source: string,
    readonly chosen: boolean,
  ) {
    super();
  }
  eq(other: WidgetType) {
    return other instanceof Drawn && other.source === this.source && other.chosen === this.chosen;
  }
  updateDOM(dom: HTMLElement) {
    if (dom.dataset.source !== this.source) return false;
    dom.classList.toggle("cm-rich-chosen", this.chosen);
    return true;
  }
  get estimatedHeight() {
    return this.source.split("\n").length * LINE_PX;
  }
  toDOM(view: EditorView) {
    const box = view.dom.ownerDocument.createElement("div");
    box.className = this.chosen ? "cm-rich cm-rich-chosen" : "cm-rich";
    box.dataset.source = this.source;
    const draw = view.state.facet(hosted).drawBlock;
    const gone = draw ? draw(this.source, box, () => view.requestMeasure()) : undefined;
    if (!draw) box.textContent = this.source;
    if (gone) leaving.set(box, gone);
    box.addEventListener("toggle", () => view.requestMeasure(), true);
    const span = () => {
      const from = view.posAtDOM(box);
      return { from, to: view.state.doc.lineAt(from + this.source.length).to };
    };
    box.addEventListener("mousedown", (event) => {
      if (event.button !== 0) return;
      const target = event.target as Element;
      if (target.closest?.("summary")) return;
      const cell = target.closest?.<HTMLTableCellElement>("td, th");
      if (cell?.isContentEditable) return;
      event.preventDefault();
      const { from, to } = span();
      const grip = target.closest?.<HTMLElement>(".col-grip");
      if (grip) return columnWidened(view, grip, event, { from, to });
      const asked = target.closest?.<HTMLElement>("[data-act]");
      if (asked?.dataset.act === "rowAdd") return rowAdded(view, this.source, from);
      if (asked?.dataset.act === "column") return columnAsked(view, asked, from);
      if (asked?.dataset.act === "row") return rowAsked(view, asked, from);
      if (asked?.dataset.act === "toggle" || asked?.dataset.act === "star")
        return cellPressed(view, asked, from);
      if (cell && box.contains(cell) && cellEdited(view, cell, this.source, from)) return;
      view.focus();
      if (event.detail >= 2 && !box.querySelector("table"))
        view.dispatch({ effects: reveal.of(from), selection: { anchor: from } });
      else view.dispatch({ selection: { anchor: from, head: to } });
    });
    return box;
  }
  destroy(dom: HTMLElement) {
    leaving.get(dom)?.();
  }
  ignoreEvent() {
    return true;
  }
}

const leaving = new WeakMap<HTMLElement, () => void>();

class Unseen extends WidgetType {
  eq(other: WidgetType) {
    return other instanceof Unseen;
  }
  toDOM(view: EditorView) {
    return view.dom.ownerDocument.createElement("div");
  }
}

// The properties at the top live in the side panel; the Markdown view still shows them as written.
const propertiesEnd = (state: EditorState): number | null => {
  const held = frontLines(state.sliceDoc(0, Math.min(state.doc.length, FRONT_AT_MOST)));
  return held ? state.doc.line(held.length).to : null;
};

const drawnAll = (state: EditorState): DecorationSet => {
  if (state.field(marksShown, false)) return Decoration.none;
  const open = state.field(revealed);
  const { from: chosenFrom, to: chosenTo } = state.selection.main;
  const out: Range<Decoration>[] = [];
  const front = propertiesEnd(state);
  if (front !== null)
    out.push(Decoration.replace({ widget: new Unseen(), block: true }).range(0, front));
  for (const one of richIn(state)) {
    if (open !== null && open >= one.from && open <= one.to) continue;
    const source = state.doc.sliceString(one.from, one.to);
    const chosen = chosenFrom <= one.from && chosenTo >= one.to && chosenFrom !== chosenTo;
    const widget = new Drawn(source, chosen);
    out.push(Decoration.replace({ widget, block: true }).range(one.from, one.to));
  }
  return Decoration.set(out);
};

const reshaped = (tr: Transaction) =>
  tr.docChanged ||
  syntaxTree(tr.state) !== syntaxTree(tr.startState) ||
  tr.effects.length > 0 ||
  tr.state.field(revealed) !== tr.startState.field(revealed);

// Rebuilding every drawn block on each caret move is what made long documents slow to walk through.
const chosenTurned = (was: DecorationSet, tr: Transaction): boolean => {
  const { from: start, to: end } = tr.state.selection.main;
  let turned = false;
  was.between(0, tr.state.doc.length, (from, to, one) => {
    const widget = one.spec.widget;
    if (!(widget instanceof Drawn)) return;
    const now = start <= from && end >= to && start !== end;
    if (now !== widget.chosen) turned = true;
  });
  return turned;
};

const drawnBlocks = StateField.define<DecorationSet>({
  create: drawnAll,
  update: (was, tr) => {
    if (reshaped(tr)) return drawnAll(tr.state);
    if (tr.selection && chosenTurned(was, tr)) return drawnAll(tr.state);
    return was;
  },
  provide: (field) => [
    EditorView.decorations.from(field),
    EditorView.atomicRanges.of((view) => view.state.field(field)),
  ],
});

// A drawn block beside the caret is taken whole first, so one key never wipes a table by surprise.
const takenFirst =
  (way: -1 | 1) =>
  (view: EditorView): boolean => {
    const { state } = view;
    const { main } = state.selection;
    if (!main.empty || state.field(marksShown, false)) return false;
    const line = state.doc.lineAt(main.head);
    if (way < 0 ? main.head !== line.from : main.head !== line.to) return false;
    let at = line.number + way;
    while (at >= 1 && at <= state.doc.lines && !state.doc.line(at).text.trim()) at += way;
    if (at < 1 || at > state.doc.lines) return false;
    const edge = state.doc.line(at);
    const near = richIn(state).find((one) =>
      way < 0 ? one.to === edge.to : one.from === edge.from,
    );
    if (!near) return false;
    view.dispatch({ selection: { anchor: near.from, head: near.to } });
    return true;
  };

// A block taken whole opens to its Markdown with Enter, as a double press does; a table is written in its cells.
const opened = (view: EditorView): boolean => {
  const { from, to } = view.state.selection.main;
  if (from === to || view.state.field(marksShown, false)) return false;
  const block = richIn(view.state).find((one) => one.from === from && one.to === to);
  if (!block || view.state.doc.sliceString(from, from + 1) === "|") return false;
  view.dispatch({ effects: reveal.of(from), selection: { anchor: from } });
  return true;
};

export const drawnKeys: KeyBinding[] = [
  { key: "Enter", run: opened },
  { key: "Backspace", run: takenFirst(-1) },
  { key: "Delete", run: takenFirst(1) },
];

// Words written before the hidden properties would end them, so the caret always lands after them.
const afterProperties = EditorState.transactionFilter.of((tr) => {
  if (!tr.selection || tr.state.field(marksShown, false)) return tr;
  const front = propertiesEnd(tr.state);
  if (front === null) return tr;
  const past = Math.min(tr.state.doc.length, front + 1);
  const held = tr.state.selection.ranges.some((one) => one.from <= front);
  if (!held) return tr;
  const kept = EditorSelection.create(
    tr.state.selection.ranges.map((one) =>
      EditorSelection.range(Math.max(one.anchor, past), Math.max(one.head, past)),
    ),
    tr.state.selection.mainIndex,
  );
  return [tr, { selection: kept, sequential: true }];
});

export const drawing = [revealed, drawnBlocks, afterProperties];
