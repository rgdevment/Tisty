import { invoke } from "@tauri-apps/api/core";

/** One change to a table; `row` counts the rows under the head from 0. Mirrors `tisty_markdown::grid::Edit`. */
export type TableEdit =
  | { readonly kind: "cell"; readonly row: number; readonly column: number; readonly said: string }
  | { readonly kind: "rename"; readonly column: number; readonly said: string }
  | { readonly kind: "retype"; readonly column: number; readonly words: string }
  | { readonly kind: "rowAdd"; readonly at: number }
  | { readonly kind: "rowRemove" | "rowCopy"; readonly row: number }
  | { readonly kind: "rowMove" | "columnMove"; readonly from: number; readonly to: number }
  | { readonly kind: "columnAdd"; readonly at: number }
  | { readonly kind: "columnRemove"; readonly column: number }
  | { readonly kind: "sort"; readonly column: number; readonly down: boolean };

export type TableRefused = "moved" | "beyond" | "unknownType" | "lastColumn";

// `at` is the table's place in bytes, as the page's `data-at` says it.
export const tableEdit = (body: string, at: [number, number], edit: TableEdit): Promise<string> =>
  invoke("table_edit", { body, at, edit });
