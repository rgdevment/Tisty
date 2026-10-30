import { Facet } from "@codemirror/state";
import type { EditorView, KeyBinding } from "@codemirror/view";
import { inside, placed, replaced, tableAt, tableHere } from "tisty-editor/edit";
import { refused, type TableEdit, tableEditNear } from "../tables";

/** What the editor asks of a table that changes what it means, not just where the caret is. */
export type Regrid =
  | { readonly kind: "row"; readonly by: -1 | 1 }
  | { readonly kind: "column"; readonly by: -1 | 1 }
  | { readonly kind: "sort"; readonly down: boolean }
  | { readonly kind: "recast"; readonly word: string };

/** Where the editor tells the window that the core refused a change to a table. */
export const gridFailed = Facet.define<(problem: unknown) => void, (problem: unknown) => void>({
  combine: (all) => all[0] ?? (() => {}),
});

const bytes = (said: string): number => new TextEncoder().encode(said).length;

// What the editor asks, as the core's edit of the table, and where the caret lands after it.
const asked = (
  how: Regrid,
  row: number,
  column: number,
  rows: number,
  columns: number,
): { edit: TableEdit; row: number; column: number } | null => {
  const body = row - 2;
  if (how.kind === "row") {
    const to = body + how.by;
    if (body < 0 || to < 0 || to >= rows) return null;
    return { edit: { kind: "rowMove", from: body, to }, row: to + 2, column };
  }
  if (how.kind === "column") {
    const to = column + how.by;
    if (to < 0 || to >= columns) return null;
    return { edit: { kind: "columnMove", from: column, to }, row, column: to };
  }
  if (how.kind === "sort") return { edit: { kind: "sort", column, down: how.down }, row, column };
  return { edit: { kind: "recast", column, word: how.word }, row, column };
};

/** Hands the table under the caret to the core and puts the answer back, unless the text moved on meanwhile. */
export const regrid = (view: EditorView, how: Regrid): boolean => {
  const table = tableHere(view.state);
  if (!table) return false;
  const text = view.state.doc.toString();
  const caret = view.state.selection.main.head;
  const { row, column } = placed(table, caret);
  const columns = table.rows[0].edges.length - 1;
  const wanted = asked(how, row, column, table.rows.length - 2, columns);
  // Inside a table Alt and an arrow never fall through to moving a line, which would tear the head or the rule away.
  if (!wanted) return true;
  // The first cell's words lie inside the table, where a quote's `>`, a list's indent or a row's end may not.
  const within = inside(table.rows[row], 0)[0];
  tableEditNear(text, bytes(text.slice(0, within)), wanted.edit)
    .then((next) => {
      if (view.state.doc.toString() !== text) return;
      const now = tableAt(next, table.from);
      const line = now?.rows[wanted.row];
      const head = line ? inside(line, wanted.column)[0] : caret;
      replaced(view, { text: next, head });
    })
    .catch((why) => view.state.facet(gridFailed)(refused(why)));
  return true;
};

// Alt and an arrow move the row or the column, as they move a line elsewhere; on a Mac, Option and a side arrow is a word, so a column takes Control too.
export const gridKeys: KeyBinding[] = [
  { key: "Alt-ArrowUp", run: (view) => regrid(view, { kind: "row", by: -1 }) },
  { key: "Alt-ArrowDown", run: (view) => regrid(view, { kind: "row", by: 1 }) },
  {
    key: "Alt-ArrowLeft",
    mac: "Ctrl-Alt-ArrowLeft",
    run: (view) => regrid(view, { kind: "column", by: -1 }),
  },
  {
    key: "Alt-ArrowRight",
    mac: "Ctrl-Alt-ArrowRight",
    run: (view) => regrid(view, { kind: "column", by: 1 }),
  },
];
