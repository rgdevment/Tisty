import type { EditorView } from "@codemirror/view";
import { hosted, type TableEdit } from "./hosted";

const cellsOf = (line: string): string[] =>
  line
    .trim()
    .replace(/^\|/, "")
    .replace(/(?<!\\)\|$/, "")
    .split(/(?<!\\)\|/)
    .map((one) => one.trim());

const rowsOf = (source: string): string[] => source.split("\n").filter((one) => one.trim());

const rawOf = (source: string, row: number, column: number): string | null => {
  const lines = rowsOf(source);
  const line = row < 0 ? lines[0] : lines[row + 2];
  return line === undefined ? null : (cellsOf(line)[column] ?? null);
};

export const applied = (view: EditorView, next: string) => {
  const now = view.state.doc.toString();
  let start = 0;
  while (start < now.length && start < next.length && now[start] === next[start]) start++;
  let end = 0;
  while (
    end < now.length - start &&
    end < next.length - start &&
    now[now.length - 1 - end] === next[next.length - 1 - end]
  )
    end++;
  view.dispatch({
    changes: { from: start, to: now.length - end, insert: next.slice(start, next.length - end) },
    userEvent: "input.table",
  });
};

export const tableEdited = (
  view: EditorView,
  from: number,
  edit: TableEdit,
  failed: () => void,
) => {
  const text = view.state.doc.toString();
  const at = new TextEncoder().encode(text.slice(0, from)).length;
  view.state
    .facet(hosted)
    .tableEditNear(text, at, edit)
    .then((next) => (view.state.doc.toString() === text ? applied(view, next) : failed()))
    .catch(failed);
};

export const rowAdded = (view: EditorView, source: string, from: number) =>
  tableEdited(view, from, { kind: "rowAdd", at: Math.max(rowsOf(source).length - 2, 0) }, () => {});

const ISO_DAY = /^\d{4}-\d{2}-\d{2}$/;

const typeOf = (cell: HTMLTableCellElement): string =>
  cell.closest("table")?.querySelectorAll<HTMLElement>("thead th")[cell.cellIndex]?.dataset.type ??
  "";

// The system's own calendar picks the day, and the cell keeps the day as the core reads it.
const dayPicked = (
  view: EditorView,
  cell: HTMLTableCellElement,
  place: { row: number; column: number; from: number },
  raw: string,
) => {
  const page = cell.ownerDocument;
  const shown = cell.innerHTML;
  const input = page.createElement("input");
  input.type = "date";
  input.className = "cm-cell-day";
  const key = cell.dataset.key ?? raw;
  input.value = ISO_DAY.test(key) ? key : "";
  cell.replaceChildren(input);
  input.focus();
  try {
    input.showPicker();
  } catch {}
  let done = false;
  const back = () => {
    if (cell.isConnected) cell.innerHTML = shown;
  };
  const finish = (said: string | null) => {
    if (done) return;
    done = true;
    if (said === null || said === key) back();
    else
      tableEdited(
        view,
        place.from,
        { kind: "cell", row: place.row, column: place.column, said },
        back,
      );
    view.focus();
  };
  input.addEventListener("change", () => finish(input.value));
  input.addEventListener("blur", () => finish(null));
  input.addEventListener("keydown", (event) => {
    if (event.key === "Escape") finish(null);
    else if (event.key === "Enter") finish(input.value);
    else return;
    event.preventDefault();
    event.stopPropagation();
  });
};

export const cellPressed = (view: EditorView, asked: HTMLElement, from: number) => {
  const row = Number(asked.dataset.row);
  const column = Number(asked.dataset.column);
  const to = Number(asked.dataset.to ?? 0);
  if (Number.isNaN(row) || Number.isNaN(column)) return;
  const said = asked.dataset.act === "toggle" ? (to ? "[x]" : "[ ]") : to ? String(to) : "";
  tableEdited(view, from, { kind: "cell", row, column, said }, () => {});
};

export const cellEdited = (
  view: EditorView,
  cell: HTMLTableCellElement,
  source: string,
  from: number,
): boolean => {
  const head = cell.tagName === "TH";
  const row = head ? -1 : Number(cell.closest("tr")?.dataset.row ?? Number.NaN);
  const column = cell.cellIndex;
  const raw = Number.isNaN(row) ? null : rawOf(source, row, column);
  if (raw === null) return false;
  if (!head && typeOf(cell) === "date") {
    dayPicked(view, cell, { row, column, from }, raw);
    return true;
  }
  const shown = cell.innerHTML;
  const page = cell.ownerDocument;
  cell.textContent = raw;
  cell.contentEditable = "plaintext-only";
  cell.classList.add("cm-cell-editing");
  cell.focus();
  page.getSelection()?.selectAllChildren(cell);
  let done = false;
  const back = () => {
    if (cell.isConnected) cell.innerHTML = shown;
  };
  const finish = (keep: boolean) => {
    if (done) return;
    done = true;
    cell.removeEventListener("keydown", keyed);
    cell.removeEventListener("blur", blurred);
    cell.contentEditable = "false";
    cell.classList.remove("cm-cell-editing");
    const said = (cell.textContent ?? "").replace(/\s*\n\s*/g, " ").trim();
    if (!keep || said === raw) back();
    else {
      const edit: TableEdit = head
        ? { kind: "rename", column, said }
        : { kind: "cell", row, column, said };
      tableEdited(view, from, edit, back);
    }
    view.focus();
  };
  const keyed = (event: KeyboardEvent) => {
    if (event.key === "Escape") finish(false);
    else if (event.key === "Tab" || (event.key === "Enter" && !event.shiftKey)) finish(true);
    else return;
    event.preventDefault();
    event.stopPropagation();
  };
  const blurred = () => finish(true);
  cell.addEventListener("keydown", keyed);
  cell.addEventListener("blur", blurred);
  return true;
};
