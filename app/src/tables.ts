import { invoke } from "@tauri-apps/api/core";
import type { TableEdit } from "tisty-editor/edit";
import { t } from "./locales";

export type { TableEdit } from "tisty-editor/edit";

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
