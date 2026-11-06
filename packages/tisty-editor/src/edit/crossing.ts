import { completionStatus } from "@codemirror/autocomplete";
import type { EditorView, KeyBinding } from "@codemirror/view";
import { LEADS } from "./barring";
import { coded } from "./formatting";
import { tableHere } from "./tabling";

export type Crossing = {
  readonly forward: boolean;
  readonly x: number | null;
  readonly join: boolean;
};

export const WRITTEN =
  ".page > :is(p, h1, h2, h3, h4, h5, h6, ul, ol, blockquote)[data-at]:not(:has(> img:only-child))";

const beside = (from: Element, forward: boolean) => {
  let one = forward ? from.nextElementSibling : from.previousElementSibling;
  while (one?.matches(".cm-hint"))
    one = forward ? one.nextElementSibling : one.previousElementSibling;
  return one;
};

const writtenPast = (host: Element, forward: boolean): boolean => {
  for (let one = beside(host, forward); one; one = beside(one, forward)) {
    if (one.matches(WRITTEN)) return true;
  }
  return false;
};

const free = (view: EditorView) =>
  view.state.selection.main.empty && completionStatus(view.state) !== "active";

const onEdgeRow = (view: EditorView, forward: boolean): boolean => {
  const { doc, selection } = view.state;
  const { head } = selection.main;
  if (doc.lineAt(head).number !== (forward ? doc.lines : 1)) return false;
  const here = view.coordsAtPos(head);
  const edge = view.coordsAtPos(forward ? doc.length : 0);
  return !here || !edge || (here.bottom > edge.top && here.top < edge.bottom);
};

const wordsStart = (view: EditorView) => LEADS.exec(view.state.doc.line(1).text)?.[0].length ?? 0;

const onEdge = (view: EditorView, forward: boolean): boolean => {
  const { head } = view.state.selection.main;
  return forward ? head === view.state.doc.length : head <= wordsStart(view);
};

export const crossingKeys = (host: Element, cross: (one: Crossing) => void): KeyBinding[] => {
  const walk = (view: EditorView, forward: boolean, keepsColumn: boolean) => {
    if (!free(view) || !(keepsColumn ? onEdgeRow : onEdge)(view, forward)) return false;
    if (!writtenPast(host, forward)) return false;
    const { head } = view.state.selection.main;
    cross({ forward, x: keepsColumn ? (view.coordsAtPos(head)?.left ?? null) : null, join: false });
    return true;
  };
  const merge = (view: EditorView, forward: boolean) => {
    const { state } = view;
    const { head } = state.selection.main;
    if (!free(view) || head !== (forward ? state.doc.length : 0)) return false;
    if (!state.doc.length) {
      if (!writtenPast(host, forward)) return false;
      cross({ forward, x: null, join: false });
      return true;
    }
    if (coded(state, head) || tableHere(state) || !beside(host, forward)?.matches(WRITTEN))
      return false;
    cross({ forward, x: null, join: true });
    return true;
  };
  return [
    { key: "ArrowDown", run: (view) => walk(view, true, true) },
    { key: "ArrowUp", run: (view) => walk(view, false, true) },
    { key: "ArrowRight", run: (view) => walk(view, true, false) },
    { key: "ArrowLeft", run: (view) => walk(view, false, false) },
    { key: "Backspace", run: (view) => merge(view, false) },
    { key: "Delete", run: (view) => merge(view, true) },
  ];
};

export const enteredAt = (view: EditorView, forward: boolean, x: number | null): number => {
  const edge = forward ? wordsStart(view) : view.state.doc.length;
  if (x === null) return edge;
  const box = view.contentDOM.getBoundingClientRect();
  const y = forward ? box.top + 4 : box.bottom - 8;
  const found = view.posAtCoords({ x, y });
  return found ?? edge;
};
