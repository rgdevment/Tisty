import { ideaOpened, type Slashed } from "tisty-editor/edit";
import { type Edit, pastFeeds } from "./pageEdits";

export type Placing =
  | { readonly edit: Edit; readonly caret: number }
  | { readonly wrap: string; readonly close: string }
  | { readonly ask: Extract<Slashed["put"], { ask: unknown }>["ask"]; readonly block: boolean };

// What already starts the line goes, so a title made a list is a list and not both.
const LEAD = /^[ \t]*(?:#{1,6}[ \t]+|[-*+][ \t]+(?:\[[ xX]\][ \t]+)?|\d+[.)][ \t]+|>[ \t]?)*/;

const lineStart = (text: string, at: number) => text.lastIndexOf("\n", at - 1) + 1;

export const blank = (text: string, at: number) => {
  const start = lineStart(text, at);
  const end = text.indexOf("\n", at);
  return !text.slice(start, end < 0 ? text.length : end).trim();
};

// A block goes on the line just begun, or after the block the caret stands in, a blank line apart.
export const blockPlaced = (
  text: string,
  at: number,
  end: number,
  block: string,
  inside: number,
) => {
  if (blank(text, at)) {
    const start = lineStart(text, at);
    const above = text.slice(0, start);
    const lead = above.trim() && !/\n[ \t]*\r?\n$/.test(above) ? "\n" : "";
    const tail = text.slice(start).startsWith("\n") || start >= text.length ? "" : "\n\n";
    return {
      edit: { from: start, to: start, insert: `${lead}${block}${tail}` },
      caret: start + lead.length + inside,
    };
  }
  const from = pastFeeds(text, end, true);
  const body = /^\r?\n/.test(text.slice(from)) ? block.replace(/\n+$/, "") : block;
  return {
    edit: { from, to: from, insert: `\n\n${body}` },
    caret: from + 2 + Math.min(inside, body.length),
  };
};

export const placed = (
  text: string,
  at: number,
  end: number,
  one: Slashed,
  first: number | null = null,
): Placing => {
  const how = "stuck" in one.put ? { block: ideaOpened(text.slice(0, at)), at: -1 } : one.put;
  if ("wrap" in how) return { wrap: how.wrap, close: how.close ?? how.wrap };
  if ("ask" in how) return { ask: how.ask, block: how.block };
  if ("inline" in how)
    return { edit: { from: at, to: at, insert: how.inline }, caret: at + how.at };
  if ("line" in how) {
    const start = blank(text, at) ? lineStart(text, at) : (first ?? lineStart(text, at));
    const lead = LEAD.exec(text.slice(start))?.[0] ?? "";
    const quoted = first !== null && how.line.startsWith(">");
    const body = quoted ? text.slice(start + lead.length, Math.max(start, end)) : "";
    const lines = body.replace(/[\r\n]+$/, "");
    const spread = quoted && /\n/.test(lines);
    const insert = spread
      ? `${how.line}${lines.replace(/\r?\n/g, (eol) => `${eol}${how.line}`)}`
      : how.line;
    const to = spread ? start + lead.length + lines.length : start + lead.length;
    const caret = spread
      ? start + insert.length
      : Math.max(start + how.line.length, at - lead.length + how.line.length);
    return { edit: { from: start, to, insert }, caret };
  }
  return blockPlaced(text, at, end, how.block, how.at < 0 ? how.block.length : how.at);
};

const BLOCK_START = /^(?:#{1,6}[ \t]|[-*+][ \t]|\d+[.)][ \t]|>|```|~~~|\|)/;
const ITEM_START = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])\s/;
const QUOTED = /^[ \t]*(?:>[ \t]?)+/;

// Words pasted that open with a heading, a list, a quote, code or a table start a block of their own, never mid-line,
// and words pasted in a quote or a note stay in it, line after line.
export const pastedAround = (text: string, from: number, to: number, said: string): string => {
  const start = lineStart(text, from);
  const before = text.slice(start, from);
  const quote = QUOTED.exec(before)?.[0] ?? "";
  const quoted = (words: string) =>
    quote ? words.replace(/\r?\n/g, (one) => `${one}${quote}`) : words;
  if (!BLOCK_START.test(said)) return quoted(said);
  const ends = text.indexOf("\n", to);
  const after = text.slice(to, ends < 0 ? text.length : ends).replace(/\r$/, "");
  const listed = ITEM_START.test(before) && ITEM_START.test(said);
  const lead = before.slice(quote.length).trim() ? (listed ? "\n" : "\n\n") : "";
  const tail = after.trim() ? "\n\n" : "";
  return quoted(`${lead}${tail ? said.replace(/\s+$/, "") : said}${tail}`);
};
