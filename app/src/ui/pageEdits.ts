export type Span = { from: number; to: number };
export type Edit = { readonly from: number; readonly to: number; readonly insert: string };

const QUOTED = /^(\s*(?:>\s?)*)/;
const LISTED = /^(\s*(?:>\s?)*\s*)([-*+]|(\d+)([.)]))(\s+)(\[[ xX]\]\s+)?/;

const lineOf = (text: string, at: number) => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const end = text.indexOf("\n", at);
  return text.slice(start, end < 0 ? text.length : end);
};

// A new paragraph keeps what the line it leaves stands in: the quote, the list and its kind of item.
export const parted = (text: string, at: number, inCode: boolean): string => {
  if (inCode) return "\n";
  const line = lineOf(text, at);
  const item = LISTED.exec(line);
  if (item) {
    const [, quote, , number, close, gap, box] = item;
    const mark = number ? `${Number(number) + 1}${close}` : item[2];
    return `\n${quote}${mark}${gap}${box ? "[ ] " : ""}`;
  }
  const quote = QUOTED.exec(line)?.[1] ?? "";
  return quote ? `\n${quote.trimEnd()}\n${quote}` : "\n\n";
};

// A change made whole, as a block moved, is laid as the one stretch of words it altered.
export const changeOf = (was: string, now: string): Edit => {
  const most = Math.min(was.length, now.length);
  let start = 0;
  while (start < most && was[start] === now[start]) start++;
  let end = 0;
  while (end < most - start && was[was.length - 1 - end] === now[now.length - 1 - end]) end++;
  return { from: start, to: was.length - end, insert: now.slice(start, now.length - end) };
};

export const unsplit = (text: string, one: Span) => ({
  from: text[one.from - 1] === "\r" && text[one.from] === "\n" ? one.from - 1 : one.from,
  to: text[one.to - 1] === "\r" && text[one.to] === "\n" ? one.to + 1 : one.to,
});

// A body written with CRLF keeps it, so a key never leaves a bare line feed among its lines.
export const linedAs = (text: string, one: Edit, caret: number): [Edit, number] => {
  if (!text.includes("\r\n") || !/(?<!\r)\n/.test(one.insert)) return [one, caret];
  const insert = one.insert.replace(/\r?\n/g, "\r\n");
  const within = Math.max(0, Math.min(caret - one.from, one.insert.length));
  const added = one.insert.slice(0, within).match(/(?<!\r)\n/g)?.length ?? 0;
  const past = caret - one.from > one.insert.length ? insert.length - one.insert.length : added;
  return [{ ...one, insert }, caret + past];
};

export const applied = (text: string, one: Edit) =>
  `${text.slice(0, one.from)}${one.insert}${text.slice(one.to)}`;
