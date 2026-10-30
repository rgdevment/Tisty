import { Facet } from "@codemirror/state";

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

// The editor holds no words and no Markdown core of its own: the window gives Tisty's, the web will give its wasm.
export type Hosted = {
  readonly say: (word: EditWord) => string;
  /** The table holding a byte of the body changed by the edits, or a refusal already in the reader's words. */
  readonly tableEditNear: (body: string, at: number, ...edits: TableEdit[]) => Promise<string>;
  readonly fromHtml: (html: string) => Promise<string>;
  readonly fromText: (text: string) => Promise<string | null>;
};

const NOBODY: Hosted = {
  say: (word) => word,
  tableEditNear: () => Promise.reject(new Error("nobody holds this editor")),
  fromHtml: () => Promise.reject(new Error("nobody holds this editor")),
  fromText: () => Promise.reject(new Error("nobody holds this editor")),
};

export const hosted = Facet.define<Hosted, Hosted>({ combine: (all) => all[0] ?? NOBODY });

// Every word the editor says, so a host that lacks one fails to build rather than showing its key.
export type EditWord =
  | "attachFile"
  | "barFold"
  | "barOpen"
  | "bigTitle"
  | "blockKind"
  | "bold"
  | "bullets"
  | "checks"
  | "codeBlock"
  | "codeSpan"
  | "diagramAsText"
  | "diagramBar"
  | "diagramExisting"
  | "editWholeDoc"
  | "expand"
  | "foldSection"
  | "formatting"
  | "formulaIt"
  | "hintInsert"
  | "hintMarks"
  | "hintWhole"
  | "hintWholeLeave"
  | "insertDoc"
  | "insertIcon"
  | "insertLabel"
  | "insertNewDoc"
  | "insertNewPage"
  | "insertPage"
  | "insertStep"
  | "italic"
  | "keptNow"
  | "leafBreak"
  | "linkIt"
  | "marksShown"
  | "midTitle"
  | "newDiagram"
  | "newDoc"
  | "newPage"
  | "numbers"
  | "paragraph"
  | "penBlue"
  | "penGreen"
  | "penIt"
  | "penPink"
  | "quote"
  | "saving"
  | "slashKeys"
  | "smallTitle"
  | "struck"
  | "table"
  | "tableAlignCenter"
  | "tableAlignLeft"
  | "tableAlignRight"
  | "tableColumnLeft"
  | "tableColumnMoveLeft"
  | "tableColumnMoveRight"
  | "tableColumnRemove"
  | "tableColumnRight"
  | "tableRowAbove"
  | "tableRowBelow"
  | "tableRowDown"
  | "tableRowRemove"
  | "tableRowUp"
  | "tableSortAsc"
  | "tableSortDesc"
  | "tableType"
  | "tableTypeCheck"
  | "tableTypeDate"
  | "tableTypeDuration"
  | "tableTypeEmail"
  | "tableTypeLink"
  | "tableTypeMoney"
  | "tableTypeNumber"
  | "tableTypePercent"
  | "tableTypePhone"
  | "tableTypeRating"
  | "tableTypeSelect"
  | "tableTypeTags"
  | "tableTypeText"
  | "tableTypeUrl"
  | "tags"
  | "underlined"
  | "untitled"
  | "wholeLeave"
  | "widgetIt"
  | `said${"note" | "tip" | "important" | "warning" | "caution"}`;
