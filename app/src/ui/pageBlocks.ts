import { ideaOpened, type Slashed } from "tisty-editor/edit";

type Edit = { readonly from: number; readonly to: number; readonly insert: string };

export type Placing =
  | { readonly edit: Edit; readonly caret: number }
  | { readonly wrap: string; readonly close: string }
  | { readonly ask: Extract<Slashed["put"], { ask: unknown }>["ask"]; readonly block: boolean };

// What already starts the line goes, so a title made a list is a list and not both.
const LEAD = /^\s*(?:#{1,6}\s+|[-*+]\s+(?:\[[ xX]\]\s+)?|\d+[.)]\s+|>\s?)*/;

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
    const tail = text.slice(start).startsWith("\n") || start >= text.length ? "" : "\n\n";
    return { edit: { from: start, to: start, insert: `${block}${tail}` }, caret: start + inside };
  }
  return { edit: { from: end, to: end, insert: `\n\n${block}` }, caret: end + 2 + inside };
};

export const placed = (text: string, at: number, end: number, one: Slashed): Placing => {
  const how = "stuck" in one.put ? { block: ideaOpened(text.slice(0, at)), at: -1 } : one.put;
  if ("wrap" in how) return { wrap: how.wrap, close: how.close ?? how.wrap };
  if ("ask" in how) return { ask: how.ask, block: how.block };
  if ("inline" in how)
    return { edit: { from: at, to: at, insert: how.inline }, caret: at + how.at };
  if ("line" in how) {
    const start = lineStart(text, at);
    const lead = LEAD.exec(text.slice(start))?.[0] ?? "";
    const caret = Math.max(start + how.line.length, at - lead.length + how.line.length);
    return { edit: { from: start, to: start + lead.length, insert: how.line }, caret };
  }
  return blockPlaced(text, at, end, how.block, how.at < 0 ? how.block.length : how.at);
};
