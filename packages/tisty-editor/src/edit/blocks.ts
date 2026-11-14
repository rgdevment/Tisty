import { ensureSyntaxTree, syntaxTree } from "@codemirror/language";
import {
  type EditorState,
  type Range,
  StateEffect,
  StateField,
  Text,
  type TransactionSpec,
} from "@codemirror/state";
import { Decoration, EditorView, type KeyBinding } from "@codemirror/view";
import type { SyntaxNode, Tree } from "@lezer/common";
import { frontLines } from "../front";
import { fencePairs } from "./seeing";

export type Block = { readonly from: number; readonly to: number; readonly level: number };
type Span = { readonly from: number; readonly to: number };

const HEADING = /^(?:ATX|Setext)Heading(\d)$/;
// Two line breaks as the document counts them, one place each whatever the file's line ends.
const BLANK = Text.of(["", "", ""]);
const LOOK_AROUND = 256;

const breaksAfter = (state: EditorState, at: number): number =>
  /^\n+/.exec(state.doc.sliceString(at, Math.min(state.doc.length, at + LOOK_AROUND)))?.[0]
    .length ?? 0;

const breaksBefore = (state: EditorState, at: number): number =>
  /\n+$/.exec(state.doc.sliceString(Math.max(0, at - LOOK_AROUND), at))?.[0].length ?? 0;
const PARSE_FOR_MS = 150;

const lineEnd = (doc: Text, at: number): number => {
  const line = doc.lineAt(at);
  return at === line.from && at > 0 ? doc.lineAt(at - 1).to : line.to;
};

const frontEnd = (state: EditorState): number => {
  const held = frontLines(state.sliceDoc(0, Math.min(state.doc.length, 8 * 1024)));
  return held ? state.doc.line(held.length).to : -1;
};

const tops = (state: EditorState, tree: Tree): Block[] => {
  const found: Block[] = [];
  const cursor = tree.topNode.cursor();
  if (!cursor.firstChild()) return found;
  do {
    const level = Number(HEADING.exec(cursor.name)?.[1] ?? 0);
    const from = state.doc.lineAt(cursor.from).from;
    found.push({ from, to: lineEnd(state.doc, cursor.to), level });
  } while (cursor.nextSibling());
  return found;
};

// A `:::` section is many paragraphs to the parser and one block to the person.
const fenced = (state: EditorState, all: Block[]): Block[] => {
  const pairs = fencePairs(state);
  if (!pairs.length) return all;
  const out: Block[] = [];
  for (const one of all) {
    const pair = pairs.find(({ open, shut }) => one.from <= shut.to && one.to >= open.from);
    const last = out[out.length - 1];
    if (pair && last && last.to >= pair.open.from && last.from <= pair.shut.to) {
      out[out.length - 1] = { from: last.from, to: Math.max(last.to, one.to), level: 0 };
      continue;
    }
    out.push(pair ? { ...one, level: 0 } : one);
  }
  return out;
};

const known = new WeakMap<Tree, Block[]>();

export const blocksOf = (state: EditorState, whole = false): Block[] => {
  const tree = whole
    ? (ensureSyntaxTree(state, state.doc.length, PARSE_FOR_MS) ?? syntaxTree(state))
    : syntaxTree(state);
  const kept = known.get(tree);
  if (kept) return kept;
  const past = frontEnd(state);
  const all = fenced(state, tops(state, tree)).filter((one) => one.from > past);
  known.set(tree, all);
  return all;
};

const indexAt = (all: Block[], at: number): number => {
  const inside = all.findIndex((one) => one.from <= at && at <= one.to);
  if (inside >= 0) return inside;
  const after = all.findIndex((one) => one.from > at);
  return after >= 0 ? after : all.length - 1;
};

const sectionEnd = (all: Block[], at: number): number => {
  const level = all[at].level;
  if (!level) return at;
  let end = at;
  while (end + 1 < all.length && !(all[end + 1].level > 0 && all[end + 1].level <= level)) end++;
  return end;
};

const itemAt = (state: EditorState, at: number): SyntaxNode | null => {
  for (let one: SyntaxNode | null = syntaxTree(state).resolveInner(at, 1); one; one = one.parent) {
    if (one.name === "ListItem") return one;
  }
  return null;
};

const itemSpan = (state: EditorState, item: SyntaxNode): Span => ({
  from: item.from,
  to: lineEnd(state.doc, item.to),
});

const sibling = (item: SyntaxNode, way: -1 | 1): SyntaxNode | null => {
  const next = way < 0 ? item.prevSibling : item.nextSibling;
  return next?.name === "ListItem" ? next : null;
};

const swapped = (state: EditorState, unit: Span, other: Span, way: -1 | 1): TransactionSpec => {
  const [first, second] = way < 0 ? [other, unit] : [unit, other];
  const gap = state.doc.slice(first.to, second.from);
  const unitText = state.doc.slice(unit.from, unit.to);
  const otherText = state.doc.slice(other.from, other.to);
  const insert =
    way < 0 ? unitText.append(gap).append(otherText) : otherText.append(gap).append(unitText);
  const start = way < 0 ? first.from : first.from + otherText.length + gap.length;
  const { anchor, head } = state.selection.main;
  const kept = (at: number) => start + Math.min(Math.max(at - unit.from, 0), unitText.length);
  return {
    changes: { from: first.from, to: second.to, insert },
    selection: { anchor: kept(anchor), head: kept(head) },
    scrollIntoView: true,
    userEvent: "move.block",
  };
};

const neighbourAbove = (all: Block[], from: number, section: boolean): Span | null => {
  if (from === 0) return null;
  if (!section) return all[from - 1];
  const level = all[from].level;
  let parent = from - 1;
  while (parent >= 0 && !(all[parent].level > 0 && all[parent].level <= level)) parent--;
  const start = parent >= 0 && all[parent].level === level ? parent : parent + 1;
  if (start >= from) return null;
  return { from: all[start].from, to: all[from - 1].to };
};

const neighbourBelow = (all: Block[], to: number, section: boolean, level: number): Span | null => {
  const next = to + 1;
  if (next >= all.length) return null;
  if (!section) return all[next];
  if (all[next].level !== level) return null;
  return { from: all[next].from, to: all[sectionEnd(all, next)].to };
};

const covering = (state: EditorState, all: Block[]): [number, number] => {
  const { from, to } = state.selection.main;
  return [indexAt(all, from), indexAt(all, to)];
};

export const unitOf = (state: EditorState, all = blocksOf(state, true)): Span | null => {
  if (!all.length) return null;
  const [first, last] = covering(state, all);
  const end = first === last ? sectionEnd(all, first) : last;
  return { from: all[first].from, to: all[end].to };
};

export const unitAt = (
  state: EditorState,
  at: number,
  whole = true,
): (Span & { readonly next: number }) | null => {
  const all = blocksOf(state, whole);
  if (!all.length) return null;
  const first = indexAt(all, at);
  const end = sectionEnd(all, first);
  return { from: all[first].from, to: all[end].to, next: all[first + 1]?.from ?? state.doc.length };
};

export const moved = (state: EditorState, way: -1 | 1): TransactionSpec | null => {
  const choosing = state.field(chosen, false) ?? false;
  const item = choosing ? null : itemAt(state, state.selection.main.head);
  if (item) {
    const other = sibling(item, way);
    return other ? swapped(state, itemSpan(state, item), itemSpan(state, other), way) : null;
  }
  const all = blocksOf(state, true);
  if (!all.length) return null;
  const [first, last] = covering(state, all);
  const section = first === last && all[first].level > 0;
  const end = section ? sectionEnd(all, first) : last;
  const unit = { from: all[first].from, to: all[end].to };
  const other =
    way < 0
      ? neighbourAbove(all, first, section)
      : neighbourBelow(all, end, section, all[first].level);
  if (!other) return null;
  const spec = swapped(state, unit, other, way);
  return choosing ? { ...spec, effects: choose.of(true) } : spec;
};

export const movedTo = (state: EditorState, unit: Span, target: number): TransactionSpec | null => {
  if (target >= unit.from && target <= unit.to) return null;
  const doc = state.doc;
  const text = doc.slice(unit.from, unit.to);
  let cutFrom = unit.from;
  let cutTo = unit.to;
  const after = breaksAfter(state, unit.to);
  if (after && unit.to + after < doc.length) cutTo = unit.to + after;
  else cutFrom = unit.from - breaksBefore(state, unit.from);
  const atEnd = target >= doc.length;
  const insert = atEnd ? BLANK.append(text) : text.append(BLANK);
  const at = atEnd ? doc.length : target;
  const start = atEnd ? at + BLANK.length : at;
  const shift = start < cutFrom ? 0 : -(cutTo - cutFrom);
  return {
    changes: [
      { from: cutFrom, to: cutTo },
      { from: at, insert },
    ],
    selection: { anchor: start + shift, head: start + shift + text.length },
    effects: choose.of(true),
    scrollIntoView: true,
    userEvent: "move.drop",
  };
};

const choose = StateEffect.define<boolean>();

export const chosen = StateField.define<boolean>({
  create: () => false,
  update: (was, tr) => {
    for (const one of tr.effects) if (one.is(choose)) return one.value;
    return was && !tr.selection && !tr.docChanged;
  },
});

const chosenLines = EditorView.decorations.compute(["selection", chosen], (state) => {
  if (!state.field(chosen)) return Decoration.none;
  const { from, to } = state.selection.main;
  const out: Range<Decoration>[] = [];
  const mark = Decoration.line({ class: "cm-chosen" });
  for (let at = state.doc.lineAt(from).number; at <= state.doc.lineAt(to).number; at++) {
    out.push(mark.range(state.doc.line(at).from));
  }
  return Decoration.set(out);
});

export const chooseAt = (view: EditorView, span: Span) =>
  view.dispatch({ selection: { anchor: span.from, head: span.to }, effects: choose.of(true) });

export const chooseOrLet = (view: EditorView): boolean => {
  const { state } = view;
  if (state.field(chosen)) {
    view.dispatch({ selection: { anchor: state.selection.main.head }, effects: choose.of(false) });
    return true;
  }
  const item = itemAt(state, state.selection.main.head);
  const span = item ? itemSpan(state, item) : unitOf(state);
  if (!span) return false;
  chooseAt(view, span);
  return true;
};

const grown = (view: EditorView, way: -1 | 1): boolean => {
  const { state } = view;
  if (!state.field(chosen)) return false;
  const all = blocksOf(state, true);
  const { anchor, head } = state.selection.main;
  const start = indexAt(all, anchor);
  const end = indexAt(all, head) + way;
  if (end < 0 || end >= all.length) return true;
  const down = end >= start;
  view.dispatch({
    selection: {
      anchor: down ? all[start].from : all[start].to,
      head: down ? all[end].to : all[end].from,
    },
    effects: choose.of(true),
    scrollIntoView: true,
  });
  return true;
};

const dropped = (view: EditorView): boolean => {
  const { state } = view;
  if (!state.field(chosen)) return false;
  const { from, to } = state.selection.main;
  const after = breaksAfter(state, to);
  const before = breaksBefore(state, from);
  const [cutFrom, cutTo] =
    after && to + after < state.doc.length ? [from, to + after] : [from - before, to];
  view.dispatch({
    changes: { from: Math.max(cutFrom, 0), to: cutTo },
    selection: { anchor: Math.max(cutFrom, 0) },
    effects: choose.of(false),
    scrollIntoView: true,
    userEvent: "delete.block",
  });
  return true;
};

export const doubled = (view: EditorView): boolean => {
  const { state } = view;
  const span = state.field(chosen) ? state.selection.main : unitOf(state);
  if (!span) return false;
  const text = state.doc.slice(span.from, span.to);
  const start = span.to + BLANK.length;
  view.dispatch({
    changes: { from: span.to, insert: BLANK.append(text) },
    selection: { anchor: start, head: start + text.length },
    effects: choose.of(true),
    scrollIntoView: true,
    userEvent: "input.copy",
  });
  return true;
};

export const moving =
  (way: -1 | 1) =>
  (view: EditorView): boolean => {
    const spec = moved(view.state, way);
    if (spec) view.dispatch(spec);
    return true;
  };

export const droppedBlock = (view: EditorView): boolean => {
  if (!view.state.field(chosen)) chooseOrLet(view);
  return dropped(view);
};

export const livingKeys: KeyBinding[] = [
  { key: "Alt-ArrowUp", run: moving(-1) },
  { key: "Alt-ArrowDown", run: moving(1) },
  { key: "Shift-ArrowUp", run: (view) => grown(view, -1) },
  { key: "Shift-ArrowDown", run: (view) => grown(view, 1) },
  { key: "Backspace", run: dropped },
  { key: "Delete", run: dropped },
  { key: "Mod-d", run: doubled },
];

export const blocking = [chosen, chosenLines];
