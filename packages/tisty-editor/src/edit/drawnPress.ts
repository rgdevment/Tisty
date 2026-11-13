import type { EditorView } from "@codemirror/view";
import { cellEdited, cellPressed, rowAdded } from "./celling";
import { codeOf, diagramsBefore } from "./drawnKinds";
import { hosted } from "./hosted";
import { ideaEdited } from "./ideaWriting";
import { columnAsked, columnWidened, rowAsked } from "./tableMenus";

// Long enough for a second press to arrive, so a double press opens the diagram's window and not its picture.
const DOUBLE_MS = 300;

type Span = { readonly from: number; readonly to: number };

const waiting = new WeakMap<HTMLElement, ReturnType<typeof setTimeout>>();

const tablePressed = (
  view: EditorView,
  target: Element,
  source: string,
  span: Span,
  event: MouseEvent,
) => {
  const grip = target.closest<HTMLElement>(".col-grip");
  if (grip) return (columnWidened(view, grip, event, span), true);
  const asked = target.closest<HTMLElement>("[data-act]");
  const act = asked?.dataset.act;
  if (asked && act === "rowAdd") return (rowAdded(view, source, span.from), true);
  if (asked && act === "column") return (columnAsked(view, asked, span.from), true);
  if (asked && act === "row") return (rowAsked(view, asked, span.from), true);
  if (asked && (act === "toggle" || act === "star"))
    return (cellPressed(view, asked, span.from), true);
  const cell = target.closest<HTMLTableCellElement>("td, th");
  return Boolean(cell && cellEdited(view, cell, source, span.from));
};

const drawingPressed = (
  view: EditorView,
  box: HTMLElement,
  target: Element,
  source: string,
  span: Span,
  event: MouseEvent,
) => {
  const asked = view.state.facet(hosted).drawnAsked;
  const button = target.closest<HTMLElement>(".diagram-bar [data-asked]");
  if (button && asked) {
    const nth = diagramsBefore(view.state, span.from);
    const kind = button.dataset.asked;
    if (kind === "diagram") asked({ kind: "diagram", code: codeOf(source), from: span.from, nth });
    if (kind === "diagramExport") asked({ kind: "export", code: codeOf(source), nth, by: button });
    return true;
  }
  const figure = target.closest<HTMLElement>("figure.diagram");
  if (figure && asked) {
    clearTimeout(waiting.get(box));
    const nth = diagramsBefore(view.state, span.from);
    if (event.detail >= 2) asked({ kind: "diagram", code: codeOf(source), from: span.from, nth });
    else
      waiting.set(
        box,
        setTimeout(() => asked({ kind: "zoom", box, target: figure }), DOUBLE_MS),
      );
    return true;
  }
  const picture = target.closest<HTMLElement>("img");
  if (picture && asked) return (asked({ kind: "zoom", box, target: picture }), true);
  return false;
};

export const drawnPressed = (
  view: EditorView,
  box: HTMLElement,
  source: string,
  span: Span,
  event: MouseEvent,
): boolean => {
  const target = event.target as Element;
  if (target.closest?.("summary")) return true;
  const cell = target.closest?.<HTMLTableCellElement>("td, th");
  if (cell?.isContentEditable || target.closest?.(".cm-idea-words")) return true;
  event.preventDefault();
  if (target.closest?.("table, .table") && tablePressed(view, target, source, span, event))
    return true;
  if (drawingPressed(view, box, target, source, span, event)) return true;
  const note = target.closest?.<HTMLElement>('blockquote[data-callout="idea"]');
  return Boolean(note && ideaEdited(view, note, source, span.from));
};
