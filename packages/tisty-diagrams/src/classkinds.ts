import type { Word } from "./words";

// The kinds a relation is named by, with their icon; the arrow written the other way round is the same kind.
export const KINDS: readonly (readonly [string, Word, string])[] = [
  ["<|--", "classInherits", "inherits"],
  ["..|>", "classRealizes", "realizes"],
  ["*--", "classComposes", "composes"],
  ["o--", "classAggregates", "aggregates"],
  ["-->", "classAssociates", "associates"],
  ["..>", "classDepends", "depends"],
  ["--", "classLinks", "links"],
  ["..", "classDashed", "links"],
];

/** The kinds of class the palette and the panel offer, by the `<<…>>` Mermaid writes them with. */
export const SORTS: readonly (readonly [string | null, Word])[] = [
  [null, "classPlain"],
  ["interface", "classInterface"],
  ["abstract", "classAbstract"],
  ["enumeration", "classEnumeration"],
];

export const VISIBILITY_WORDS: readonly (readonly [string, Word])[] = [
  ["", "classNoVisibility"],
  ["+", "classPublic"],
  ["-", "classPrivate"],
  ["#", "classProtected"],
  ["~", "classPackage"],
];
