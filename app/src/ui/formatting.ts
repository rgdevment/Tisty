import { syntaxTree } from "@codemirror/language";
import type { ChangeSpec, EditorState } from "@codemirror/state";
import type { EditorView, KeyBinding } from "@codemirror/view";
import { pend, pending, settled } from "./pending";

const CODE = new Set(["InlineCode", "FencedCode", "CodeBlock"]);

export const coded = (state: EditorState, at: number): boolean => {
  for (let node = syntaxTree(state).resolveInner(at, 1); node.parent; node = node.parent) {
    if (CODE.has(node.name)) return true;
  }
  return false;
};

export const PEN = /==(?=\S)((?:(?!==).)+?)(?<=\S)==(?:\{([a-z]+)\})?/u;
export const UNDER = /(?<![\p{L}\p{N}+])\+\+(?=\S)(.+?)(?<=\S)\+\+(?![\p{L}\p{N}+])/u;
export const NAMED = /(?<![\p{L}\p{N}])::([a-z0-9-]+)(?: ([a-z]+))?::(?![\p{L}\p{N}])/u;
export const owning = () => new RegExp(`${PEN.source}|${UNDER.source}|${NAMED.source}`, "gu");

const FORMAT: Record<string, string> = {
  "**": "StrongEmphasis",
  "*": "Emphasis",
  "~~": "Strikethrough",
  "`": "InlineCode",
  "[": "Link",
};

// What starts a line and is never part of its words: quotes, titles, items and boxes.
const LEAD = /^(?:\s*(?:>\s?|#{1,6}\s|[-*+]\s(?:\[[ xX]\]\s)?|\d+[.)]\s))*/;
const HEAD = /^\[![a-z]+\]\s*$/i;

type Span = {
  readonly from: number;
  readonly to: number;
  readonly open: readonly [number, number];
  readonly shut: readonly [number, number];
  readonly tail: string;
};

// The spans of this format that touch from..to, edges included.
const spans = (state: EditorState, mark: string, from: number, to: number): Span[] => {
  const found: Span[] = [];
  const kind = FORMAT[mark];
  if (kind) {
    syntaxTree(state).iterate({
      from,
      to,
      enter: (node) => {
        if (node.name !== kind || node.from > to || node.to < from) return;
        const open = node.node.firstChild;
        const shut = kind === "Link" ? node.node.getChildren("LinkMark")[1] : node.node.lastChild;
        if (!open || !shut || open.from === shut.from) return;
        const end = kind === "Link" ? node.to : shut.to;
        found.push({
          from: node.from,
          to: end,
          open: [open.from, open.to],
          shut: [shut.from, end],
          tail: "",
        });
      },
    });
    return found;
  }
  const look = mark === "==" ? PEN : mark === "++" ? UNDER : undefined;
  if (!look) return found;
  for (let at = state.doc.lineAt(from).number; at <= state.doc.lineAt(to).number; at++) {
    const line = state.doc.line(at);
    for (const one of line.text.matchAll(new RegExp(look.source, "gu"))) {
      const start = line.from + (one.index ?? 0);
      const end = start + one[0].length;
      if (start > to || end < from) continue;
      const body = start + 2 + one[1].length;
      found.push({
        from: start,
        to: end,
        open: [start, start + 2],
        shut: [body, end],
        tail: state.sliceDoc(body, end),
      });
    }
  }
  return found;
};

// The words of each line the selection holds, past its lead and without the spaces at its edges.
const pieces = (state: EditorState, from: number, to: number) => {
  const out: { from: number; to: number }[] = [];
  for (let at = state.doc.lineAt(from).number; at <= state.doc.lineAt(to).number; at++) {
    const line = state.doc.line(at);
    const lead = LEAD.exec(line.text)?.[0].length ?? 0;
    if (HEAD.test(line.text.slice(lead))) continue;
    const start = Math.max(from, line.from + lead);
    const end = Math.min(to, line.to);
    if (end <= start) continue;
    const words = state.sliceDoc(start, end);
    const a = start + (words.length - words.trimStart().length);
    const b = end - (words.length - words.trimEnd().length);
    if (b > a) out.push({ from: a, to: b });
  }
  return out;
};

const off = (span: Span, swap?: string): ChangeSpec[] => [
  { from: span.open[0], to: span.open[1], insert: swap ? "==" : "" },
  { from: span.shut[0], to: span.shut[1], insert: swap ?? "" },
];

// Formats this piece reaches are joined into one, so marks never cross.
const on = (piece: { from: number; to: number }, touching: Span[], mark: string, close: string) => {
  const start = Math.min(piece.from, ...touching.map((one) => one.from));
  const end = Math.max(piece.to, ...touching.map((one) => one.to));
  return [
    ...touching.flatMap((one) => off(one)),
    { from: start, insert: mark },
    { from: end, insert: close },
  ];
};

const empty = (view: EditorView, mark: string, close: string): boolean => {
  const { state } = view;
  const at = state.selection.main.head;
  const waiting = state.field(pending, false);
  if (waiting && at >= waiting.from && at <= waiting.to && waiting.mark === mark) {
    settled(view, waiting.close === close ? undefined : close);
    return true;
  }
  if (mark !== "`" && coded(state, at)) return true;
  const held = spans(state, mark, at, at)[0];
  if (held) {
    const swap = mark === "==" && held.tail !== close ? close : undefined;
    view.dispatch({ changes: off(held, swap), userEvent: "input.format" });
    return true;
  }
  const lead = LEAD.exec(state.doc.lineAt(at).text)?.[0].length ?? 0;
  const from = Math.max(at, state.doc.lineAt(at).from + lead);
  view.dispatch({
    changes: { from, insert: `${mark}${close}` },
    selection: { anchor: from + mark.length },
    effects:
      mark === "["
        ? []
        : pend.of({ from: from + mark.length, to: from + mark.length, mark, close }),
    userEvent: "input.format",
  });
  return true;
};

export const wrapped = (view: EditorView, mark: string, close = mark): boolean => {
  const { state } = view;
  const main = state.selection.main;
  if (main.empty) return empty(view, mark, close);
  const chosen = pieces(state, main.from, main.to).filter(
    (one) => mark === "`" || !coded(state, one.from),
  );
  if (!chosen.length) return true;
  const reached = chosen.map((one) => spans(state, mark, one.from, one.to));
  const held = chosen.every((one, at) =>
    reached[at].some((span) => span.from <= one.from && span.to >= one.to),
  );
  if (held) {
    const swap = mark === "==" ? reached.flat().find((one) => one.tail !== close) : undefined;
    const gone = new Set<number>();
    const changes = reached.flat().flatMap((one) => {
      if (gone.has(one.from)) return [];
      gone.add(one.from);
      return off(one, swap ? close : undefined);
    });
    view.dispatch({ changes, userEvent: "input.format" });
    return true;
  }
  if (chosen.length === 1 && !reached[0].length) {
    const [one] = chosen;
    const inside = state.sliceDoc(one.from, one.to);
    const link = mark === "[";
    const after = one.from + mark.length + inside.length;
    view.dispatch({
      changes: { from: one.from, to: one.to, insert: `${mark}${inside}${close}` },
      selection: link ? { anchor: after + 2 } : { anchor: one.from + mark.length, head: after },
      userEvent: "input.format",
    });
    return true;
  }
  if (mark === "[") return true;
  const changes = chosen.flatMap((one, at) => on(one, reached[at], mark, close));
  view.dispatch({ changes, userEvent: "input.format" });
  return true;
};

// From a link's words, the right arrow steps over the bracket into the address still to be written.
export const linking: KeyBinding = {
  key: "ArrowRight",
  run: (view) => {
    const { main } = view.state.selection;
    if (!main.empty || view.state.sliceDoc(main.head, main.head + 3) !== "]()") return false;
    view.dispatch({ selection: { anchor: main.head + 2 } });
    return true;
  },
};
