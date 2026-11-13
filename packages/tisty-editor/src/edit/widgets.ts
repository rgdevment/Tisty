import { syntaxTree } from "@codemirror/language";
import {
  EditorSelection,
  EditorState,
  type Range,
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
import type { Tree } from "@lezer/common";
import { frontLines } from "../front";
import { blocksDrawn, type Code, type Rich } from "./drawnKinds";
import { drawnPressed } from "./drawnPress";
import { hosted } from "./hosted";
import { fencePairs, marksShown } from "./seeing";

const LINE_PX = 26;
const TURN_PX = 50;
const FRONT_AT_MOST = 8 * 1024;

const foldsIn = (state: EditorState): Rich[] =>
  fencePairs(state)
    .filter(({ open }) => state.doc.sliceString(open.from, open.to).startsWith(":::details"))
    .map(({ open, shut }) => ({
      from: state.doc.lineAt(open.from).from,
      to: shut.to,
      kind: "drawn" as const,
    }));

type Shown = { readonly rich: Rich[]; readonly code: Code[] };

const shownKnown = new WeakMap<Tree, Shown>();

export const shownIn = (state: EditorState): Shown => {
  const tree = syntaxTree(state);
  const kept = shownKnown.get(tree);
  if (kept) return kept;
  const folds = foldsIn(state);
  const folded = (at: number) => folds.some((one) => one.from <= at && at <= one.to);
  const { rich, code } = blocksDrawn(state);
  const shown = {
    rich: [...rich.filter((one) => !folded(one.from)), ...folds].sort((a, b) => a.from - b.from),
    code: code.filter((one) => !folded(one.open.from)),
  };
  shownKnown.set(tree, shown);
  return shown;
};

const leaving = new WeakMap<HTMLElement, () => void>();

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
    box.addEventListener("mousedown", (event) => {
      if (event.button !== 0) return;
      const from = view.posAtDOM(box);
      const span = { from, to: view.state.doc.lineAt(from + this.source.length).to };
      if (drawnPressed(view, box, this.source, span, event)) return;
      view.focus();
      view.dispatch({ selection: { anchor: span.from, head: span.to } });
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

class Turn extends WidgetType {
  constructor(readonly chosen: boolean) {
    super();
  }
  eq(other: WidgetType) {
    return other instanceof Turn && other.chosen === this.chosen;
  }
  get estimatedHeight() {
    return TURN_PX;
  }
  toDOM(view: EditorView) {
    const page = view.dom.ownerDocument;
    const box = page.createElement("div");
    box.className = this.chosen ? "cm-turn cm-rich-chosen" : "cm-turn";
    const cut = page.createElement("div");
    cut.className = "cm-turn-cut";
    const said = view.state.facet(hosted).say("leafBreak");
    cut.textContent = said;
    box.title = said;
    box.append(cut);
    box.setAttribute("role", "separator");
    box.addEventListener("mousedown", (event) => {
      if (event.button !== 0) return;
      event.preventDefault();
      const from = view.posAtDOM(box);
      view.focus();
      view.dispatch({ selection: { anchor: from, head: view.state.doc.lineAt(from).to } });
    });
    return box;
  }
  ignoreEvent() {
    return true;
  }
}

class Unseen extends WidgetType {
  eq(other: WidgetType) {
    return other instanceof Unseen;
  }
  toDOM(view: EditorView) {
    return view.dom.ownerDocument.createElement("div");
  }
}

class Fence extends WidgetType {
  constructor(
    readonly said: string,
    readonly opening: boolean,
  ) {
    super();
  }
  eq(other: WidgetType) {
    return other instanceof Fence && other.said === this.said && other.opening === this.opening;
  }
  toDOM(view: EditorView) {
    const page = view.dom.ownerDocument;
    const box = page.createElement("div");
    box.className = this.opening ? "cm-code-head" : "cm-code-foot";
    const inner = page.createElement("div");
    inner.textContent = this.said;
    box.append(inner);
    return box;
  }
  ignoreEvent() {
    return false;
  }
}

const TITLE = /title="((?:[^"\\]|\\.)*)"/;

const saidOf = (said: string): string => {
  const title = TITLE.exec(said)?.[1]?.replace(/\\"/g, '"');
  const language = said.split(/\s+/)[0] ?? "";
  return title
    ? `${title}${language && !language.startsWith("title=") ? ` · ${language}` : ""}`
    : language;
};

// The properties at the top live in the side panel; the Markdown view still shows them as written.
const propertiesEnd = (state: EditorState): number | null => {
  const held = frontLines(state.sliceDoc(0, Math.min(state.doc.length, FRONT_AT_MOST)));
  return held ? state.doc.line(held.length).to : null;
};

const codeLine = Decoration.line({ class: "cm-code" });

const codeMarks = (state: EditorState, one: Code, out: Range<Decoration>[]) => {
  const first = state.doc.lineAt(one.open.from).number;
  const last = one.shut ? state.doc.lineAt(one.shut.from).number : state.doc.lines + 1;
  if (last - first < 2) return;
  out.push(
    Decoration.replace({ widget: new Fence(saidOf(one.said), true), block: true }).range(
      one.open.from,
      one.open.to,
    ),
  );
  for (let at = first + 1; at < last; at++) out.push(codeLine.range(state.doc.line(at).from));
  if (one.shut)
    out.push(
      Decoration.replace({ widget: new Fence("", false), block: true }).range(
        one.shut.from,
        one.shut.to,
      ),
    );
};

const isChosen = (state: EditorState, one: { from: number; to: number }) => {
  const { from, to } = state.selection.main;
  return from <= one.from && to >= one.to && from !== to;
};

const widgetOf = (state: EditorState, one: Rich): WidgetType => {
  if (one.kind === "unseen") return new Unseen();
  if (one.kind === "turn") return new Turn(isChosen(state, one));
  return new Drawn(state.doc.sliceString(one.from, one.to), isChosen(state, one));
};

const drawnAll = (state: EditorState): DecorationSet => {
  if (state.field(marksShown, false)) return Decoration.none;
  const out: Range<Decoration>[] = [];
  const front = propertiesEnd(state);
  if (front !== null)
    out.push(Decoration.replace({ widget: new Unseen(), block: true }).range(0, front));
  const { rich, code } = shownIn(state);
  for (const one of rich)
    if (front === null || one.from > front)
      out.push(
        Decoration.replace({ widget: widgetOf(state, one), block: true }).range(one.from, one.to),
      );
  for (const one of code) if (front === null || one.open.from > front) codeMarks(state, one, out);
  return Decoration.set(out, true);
};

const reshaped = (tr: Transaction) =>
  tr.docChanged || syntaxTree(tr.state) !== syntaxTree(tr.startState) || tr.effects.length > 0;

// Rebuilding every drawn block on each caret move is what made long documents slow to walk through.
const chosenTurned = (was: DecorationSet, tr: Transaction): boolean => {
  let turned = false;
  was.between(0, tr.state.doc.length, (from, to, one) => {
    const widget = one.spec.widget;
    if (!(widget instanceof Drawn || widget instanceof Turn)) return;
    if (isChosen(tr.state, { from, to }) !== widget.chosen) turned = true;
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
    const near = shownIn(state).rich.find(
      (one) => one.kind !== "unseen" && (way < 0 ? one.to === edge.to : one.from === edge.from),
    );
    if (!near) return false;
    view.dispatch({ selection: { anchor: near.from, head: near.to } });
    return true;
  };

export const drawnKeys: KeyBinding[] = [
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

export const drawing = [drawnBlocks, afterProperties];
