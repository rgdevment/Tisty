type Edit = { readonly from: number; readonly to: number; readonly insert: string };
type Moved = { readonly edit: Edit; readonly caret: number };

const ITEM = /^((?:\s*>\s?)*)(\s*)((?:[-*+]|\d+[.)])\s+)/;
const ITEM_START = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])\s/;
const EMPTY_ITEM = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])(?:\s+(?:\[[ xX]\]\s*)?)?$/;
const EMPTY_QUOTE = /^\s*(?:>\s?)+$/;

export const isItem = (line: string) => ITEM_START.test(line);

const lineAt = (text: string, at: number) => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const ends = text.indexOf("\n", at);
  return { start, line: text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "") };
};

const indentOf = (line: string) => {
  const found = ITEM.exec(line);
  return found ? { quote: found[1], lead: found[2].length, mark: found[3].length } : null;
};

const parentIndent = (text: string, start: number, quote: string, lead: number) => {
  for (const one of text.slice(0, start).split("\n").reverse()) {
    const it = indentOf(one.replace(/\r$/, ""));
    if (it && it.quote === quote && it.lead < lead) return it.lead;
  }
  return 0;
};

const lineBefore = (text: string, start: number) =>
  (
    text
      .slice(0, Math.max(0, start - 1))
      .split("\n")
      .pop() ?? ""
  ).replace(/\r$/, "");

const newlines = (run: string | undefined) => (run?.match(/\n/g) ?? []).length;

export const opening = (text: string, at: number, seed: string) => {
  const before = text.slice(0, at);
  if (!before.trim()) return "";
  const lines = before.replace(/\s+$/, "").split("\n");
  const joins = isItem(seed) && isItem(lines[lines.length - 1] ?? "");
  const have = newlines(/(?:\r?\n)*$/.exec(before)?.[0]);
  return "\n".repeat(Math.max(0, (joins ? 1 : 2) - have));
};

// An item begun past the blank line that ends a list goes back to the list, so the list stays tight.
export const joinedAt = (text: string, at: number, seed: string) => {
  const blank = /(?:\r?\n){2,}$/.exec(text.slice(0, at))?.[0];
  if (!blank || !isItem(seed)) return at;
  const lines = text.slice(0, at - blank.length).split("\n");
  if (!isItem(lines[lines.length - 1] ?? "")) return at;
  return at - blank.length + (blank.startsWith("\r") ? 2 : 1);
};

export const closing = (text: string, at: number) => {
  const after = text.slice(at);
  if (!after.trim()) return "";
  return "\n".repeat(Math.max(0, 2 - newlines(/^(?:\r?\n)*/.exec(after)?.[0])));
};

export const outdented = (text: string, at: number): Moved | null => {
  const { start, line } = lineAt(text, at);
  const it = indentOf(line);
  if (!it?.lead || at < start + line.length || !EMPTY_ITEM.test(line)) return null;
  const from = start + it.quote.length;
  const to = parentIndent(text, start, it.quote, it.lead);
  return {
    edit: { from, to: from + it.lead, insert: " ".repeat(to) },
    caret: start + line.length - it.lead + to,
  };
};

export const nested = (text: string, at: number, out: boolean): Moved | null => {
  const { start, line } = lineAt(text, at);
  const it = indentOf(line);
  if (!it) return null;
  const from = start + it.quote.length;
  const above = indentOf(lineBefore(text, start));
  const under = above && above.quote === it.quote ? above.lead + above.mark : it.lead;
  const to = out ? parentIndent(text, start, it.quote, it.lead) : under;
  if (out ? to === it.lead : to <= it.lead) return null;
  return {
    edit: { from, to: from + it.lead, insert: " ".repeat(to) },
    caret: Math.max(from, at + to - it.lead),
  };
};

export const leftBehind = (text: string, at: number): { from: number; to: number } | null => {
  const { start, line } = lineAt(text, at);
  if (at < start + line.length || !(EMPTY_ITEM.test(line) || EMPTY_QUOTE.test(line))) return null;
  const above = text.slice(0, Math.max(0, start - 1));
  const before = above.lastIndexOf("\n") + 1;
  const bare = EMPTY_QUOTE.test(line) && EMPTY_QUOTE.test(above.slice(before).replace(/\r$/, ""));
  return { from: bare ? before : start, to: start + line.length };
};
