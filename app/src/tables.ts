import { invoke } from "@tauri-apps/api/core";
import { t } from "./locales";

/** One change to a table; `row` counts the rows under the head from 0. Mirrors `tisty_markdown::grid::Edit`. */
export type TableEdit =
  | { readonly kind: "cell"; readonly row: number; readonly column: number; readonly said: string }
  | { readonly kind: "rename"; readonly column: number; readonly said: string }
  | { readonly kind: "retype"; readonly column: number; readonly words: string }
  | { readonly kind: "recast"; readonly column: number; readonly word: string }
  | { readonly kind: "rowAdd"; readonly at: number }
  | { readonly kind: "rowRemove" | "rowCopy"; readonly row: number }
  | { readonly kind: "rowMove" | "columnMove"; readonly from: number; readonly to: number }
  | { readonly kind: "columnAdd"; readonly at: number }
  | { readonly kind: "columnRemove"; readonly column: number }
  | { readonly kind: "sort"; readonly column: number; readonly down: boolean };

export type TableRefused = "moved" | "beyond" | "unknownType" | "lastColumn";

// `at` is the table's place in bytes, as the page's `data-at` says it; the edits land one after another.
export const tableEdit = (
  body: string,
  at: [number, number],
  ...edits: TableEdit[]
): Promise<string> => invoke("table_edit", { body, at, edits });

// The table holding a byte of the body, the caret's, changed by the edits one after another.
export const tableEditNear = (body: string, at: number, ...edits: TableEdit[]): Promise<string> =>
  invoke("table_edit_near", { body, at, edits });

/** A refusal from the core, in the words the window shows. */
export const refused = (why: unknown): Error =>
  new Error(
    why === "moved"
      ? t("tableRefusedMoved")
      : why === "lastColumn"
        ? t("tableRefusedLastColumn")
        : t("tableRefused"),
  );

export const tableOrder = (
  body: string,
  at: [number, number],
  column: number,
  down: boolean,
): Promise<number[]> => invoke("table_order", { body, at, column, down });

// What was typed, in the form its column's type is written in the file; a date may be said as it is spoken.
export const tableCanon = (words: string, typed: string): Promise<string> =>
  invoke("table_canon", { words, typed, locale: navigator.language });

export type TableWritten = { readonly heads: string[]; readonly rows: string[][] };

export const tableWritten = (body: string, at: [number, number]): Promise<TableWritten> =>
  invoke("table_written", { body, at });
