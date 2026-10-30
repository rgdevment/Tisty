// Every word the sheet says while it is read, so a host that lacks one fails to build rather than showing its key.
export type ReadWord =
  | "cardAway"
  | "cardHere"
  | "cardGone"
  | "kindDoc"
  | "untitledDoc"
  | "docItself"
  | "goneDoc"
  | "leafOfMany"
  | "table"
  | "tableStars"
  | "tableTypeCheck";

/** A word in the reader's language, with the values it names filled in, in its own order. */
export type Say = (word: ReadWord, ...filled: string[]) => string;
