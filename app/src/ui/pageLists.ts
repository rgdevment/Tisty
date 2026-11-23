import type { Edit } from "./pageEdits";

type Moved = { readonly edit: Edit; readonly caret: number };

const ITEM = /^((?:\s*>)*(?:(?<=>)\s)?)(\s*)((?:[-*+]|\d+[.)])\s+)/;
const ITEM_START = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])\s/;
const EMPTY_ITEM = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])(?:\s+(?:\[[ xX]\]\s*)?)?$/;
const EMPTY_QUOTE = /^\s*(?:>\s?)+$/;
const QUOTED = /^(\s*(?:>\s?)*)/;
const LISTED = /^(\s*(?:>\s?)*\s*)([-*+]|(\d+)([.)]))(\s+)(\[[ xX]\]\s+)?/;

export const isItem = (line: string) => ITEM_START.test(line);

const lineAt = (text: string, at: number) => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const ends = text.indexOf("\n", at);
  return { start, line: text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "") };
};

function* linesAbove(text: string, start: number) {
  for (let end = start - 1; end >= 0; ) {
    const from = end > 0 ? text.lastIndexOf("\n", end - 1) + 1 : 0;
    yield text.slice(from, end).replace(/\r$/, "");
    end = from - 1;
  }
}

const indentOf = (line: string) => {
  const found = ITEM.exec(line);
  if (!found) return null;
  const quote = found[1];
  const quotes = quote.split(">").length - 1;
  return { quote, quotes, lead: found[2].length, mark: found[3].length };
};

type Indent = NonNullable<ReturnType<typeof indentOf>>;

const parentIndent = (text: string, start: number, it: Indent) => {
  for (const one of linesAbove(text, start)) {
    const above = indentOf(one);
    if (above && above.quotes === it.quotes && above.lead < it.lead) return above.lead;
  }
  return 0;
};

// The items nested deeper in the one above are passed over, so Tab goes down a single level.
const sameLevelAbove = (text: string, start: number, it: Indent) => {
  for (const one of linesAbove(text, start)) {
    const above = indentOf(one);
    if (!above || above.quotes !== it.quotes) return null;
    if (above.lead <= it.lead) return above;
  }
  return null;
};

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
  const to = parentIndent(text, start, it);
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
  const above = sameLevelAbove(text, start, it);
  const under = above ? above.lead + above.mark : it.lead;
  const to = out ? parentIndent(text, start, it) : under;
  if (out ? to === it.lead : to <= it.lead) return null;
  return {
    edit: { from, to: from + it.lead, insert: " ".repeat(to) },
    caret: Math.max(from, at + to - it.lead),
  };
};

// A new paragraph keeps what the line it leaves stands in: the quote, the list and its kind of item.
export const parted = (text: string, at: number, inCode: boolean): string => {
  if (inCode) return "\n";
  const { line } = lineAt(text, at);
  const item = LISTED.exec(line);
  if (item) {
    const [, quote, , number, close, gap, box] = item;
    const mark = number ? `${Number(number) + 1}${close}` : item[2];
    return `\n${quote}${mark}${gap}${box ? "[ ] " : ""}`;
  }
  const quote = QUOTED.exec(line)?.[1] ?? "";
  return quote ? `\n${quote.trimEnd()}\n${quote}` : "\n\n";
};

const LEAD = /^[ \t]*(?:>[ \t]?)*[ \t]*/;

export const hardBreak = (text: string, at: number) => {
  const { line } = lineAt(text, at);
  if (HEADED.test(line)) return "\n";
  const item = LISTED.exec(line);
  const lead = item
    ? `${item[1]}${" ".repeat((item[2]?.length ?? 0) + (item[5]?.length ?? 0))}`
    : (LEAD.exec(line)?.[0] ?? "");
  return `\\\n${lead}`;
};

export const leftBehind = (text: string, at: number): { from: number; to: number } | null => {
  const { start, line } = lineAt(text, at);
  if (at < start + line.length || !(EMPTY_ITEM.test(line) || EMPTY_QUOTE.test(line))) return null;
  const above = text.slice(0, Math.max(0, start - 1));
  const before = above.lastIndexOf("\n") + 1;
  const bare = EMPTY_QUOTE.test(line) && EMPTY_QUOTE.test(above.slice(before).replace(/\r$/, ""));
  return { from: bare ? before : start, to: start + line.length };
};

const wordless = (line: string) => line.replace(/^[\s>]*/, "") === "";

const freed = (
  text: string,
  [start, line]: readonly [number, string],
  [lead, marks]: readonly [string, number],
  apartFrom: (next: string) => boolean,
): Moved => {
  const ends = text.indexOf("\n", start);
  const next = ends < 0 ? null : lineAt(text, ends + 1).line;
  const above = start > 0 && !wordless(lineAt(text, start - 1).line) ? `${lead.trimEnd()}\n` : "";
  const below = next !== null && apartFrom(next) ? `\n${lead.trimEnd()}` : "";
  const insert = `${above}${lead}${line.slice(marks)}${below}`;
  return {
    edit: { from: start, to: start + line.length, insert },
    caret: start + above.length + lead.length,
  };
};

// Backspace at the start of an item's words takes its bullet or its box away first, and the words stay.
export const unmarked = (text: string, at: number): Moved | null => {
  const { start, line } = lineAt(text, at);
  const found = LISTED.exec(line);
  if (!found || at !== start + found[0].length) return null;
  const box = found[6]?.length ?? 0;
  if (box) {
    const from = start + found[0].length - box;
    return { edit: { from, to: start + found[0].length, insert: "" }, caret: from };
  }
  // Words freed of their mark would read as part of the item above, or of a numbered one below, without a blank line.
  const numbered = (next: string) => LISTED.exec(next)?.[3] !== undefined;
  return freed(text, [start, line], [found[1] ?? "", found[0].length], numbered);
};

const HEADED = /^([ \t]*(?:>[ \t]?)*[ \t]*)#{1,6}[ \t]+/;
const QUOTE_MARK = /^([ \t]*(?:>[ \t]?)*)>[ \t]*/;

const UNDERLINE = /^[ \t]*(?:>[ \t]?)*[ \t]*(?:=+|-+)[ \t]*$/;

const underlined = (text: string, at: number): Moved | null => {
  const { start, line } = lineAt(text, at);
  const lead = QUOTED.exec(line)?.[1] ?? "";
  const ends = text.indexOf("\n", start);
  const under = ends < 0 ? null : lineAt(text, ends + 1).line;
  if (at !== start + lead.length || under === null || !UNDERLINE.test(under)) return null;
  const past = text.indexOf("\n", ends + 1);
  const next = past < 0 ? null : lineAt(text, past + 1).line;
  const below = next !== null && !wordless(next) ? `\n${lead.trimEnd()}` : "";
  return {
    edit: { from: start + line.length, to: ends + 1 + under.length, insert: below },
    caret: at,
  };
};

export const unlined = (text: string, at: number): Moved | null => {
  const setext = underlined(text, at);
  if (setext) return setext;
  const { start, line } = lineAt(text, at);
  const found = HEADED.exec(line) ?? QUOTE_MARK.exec(line);
  if (!found || at !== start + found[0].length) return null;
  const worded = (next: string) => !wordless(next);
  return freed(text, [start, line], [found[1] ?? "", found[0].length], worded);
};

const FENCE = /^((?:[ \t]*>)*)[ \t]*(`{3,}|~{3,})(.*)$/;
const IN_QUOTE = /^[ \t]*>/;
const STANDS_ALONE =
  /^[ \t]*(?:>[ \t]?)*[ \t]*(?:#{1,6}(?:[ \t]|$)|`{3,}|~{3,}|\||(?:-[ \t]*){3,}$|(?:\*[ \t]*){3,}$|(?:_[ \t]*){3,}$|=+[ \t]*$)/;

// A fence closes only on a run of its own mark at least as long, or where the quote it opened in ends.
const inFence = (text: string, at: number) => {
  let open: { mark: string; quoted: boolean } | null = null;
  for (const raw of text.slice(0, at).split("\n")) {
    const line = raw.replace(/\r$/, "");
    if (open?.quoted && !IN_QUOTE.test(line)) open = null;
    const found = FENCE.exec(line);
    if (!found) continue;
    const [, quotes, mark, rest] = found;
    if (open === null) {
      if (mark[0] !== "`" || !rest.includes("`")) open = { mark, quoted: quotes !== "" };
    } else if (mark[0] === open.mark[0] && mark.length >= open.mark.length && !rest.trim())
      open = null;
  }
  return open !== null;
};

const paragraphLine = (line: string) =>
  !LISTED.test(line) && !wordless(line) && !STANDS_ALONE.test(line);

// The numbered line was already words of a paragraph when the lines over it, up to a blank one, held no item.
const heldBefore = (text: string, at: number) => {
  let over = 0;
  for (const one of linesAbove(text, text.indexOf("\n", at) + 1)) {
    if (LISTED.test(one)) return false;
    if (!paragraphLine(one)) return over > 0;
    over += 1;
  }
  return over > 0;
};

// Words joined across a line end would take a numbered item under them, which cannot open a list there, as their own.
export const joinedApart = (
  text: string,
  asked: Edit,
  at = asked.from + asked.insert.length,
): [Edit, number] => {
  const kept: [Edit, number] = [asked, at];
  if (!text.slice(asked.from, asked.to).includes("\n")) return kept;
  const landed = asked.from + asked.insert.length;
  const left = text.slice(0, asked.from) + asked.insert + text.slice(asked.to);
  if (landed === 0 || left[landed - 1] === "\n") return kept;
  const { start, line } = lineAt(left, landed);
  const ends = left.indexOf("\n", landed);
  const item = ends < 0 ? null : LISTED.exec(lineAt(left, ends + 1).line);
  const number = item?.[3];
  const deep = (item?.[1].replace(/^.*>/, "").replace(/\t/g, "    ").length ?? 0) >= 4;
  if (
    !item ||
    number === undefined ||
    number === "1" ||
    deep ||
    !paragraphLine(line) ||
    heldBefore(text, asked.to) ||
    inFence(left, ends)
  )
    return kept;
  const end = asked.to + (start + line.length - landed);
  const apart = `\n${item[1].trimEnd()}`;
  const split = start + line.length;
  return [
    {
      from: asked.from,
      to: end,
      insert: `${asked.insert}${text.slice(asked.to, end)}${apart}`,
    },
    at > split ? at + apart.length : at,
  ];
};

export const openedAbove = (
  text: string,
  at: number,
): { at: number; seed: string; tail: string } | null => {
  const { start, line } = lineAt(text, at);
  if (!line.trim()) return null;
  const item = LISTED.exec(line);
  const marks = item?.[0] ?? /^[ \t]*(?:>[ \t]?)*[ \t]*(?:#{1,6}[ \t]+)?/.exec(line)?.[0] ?? "";
  if (at !== start + marks.length || !line.slice(marks.length).trim()) return null;
  // A line opened inside a quote or a note stays in it, or it would split the note in two.
  const quote = /^[ \t]*(?:>[ \t]?)+/.exec(line)?.[0] ?? "";
  if (!item) return { at: start, seed: quote, tail: quote ? `\n${quote.trimEnd()}\n` : "\n\n" };
  return { at: start, seed: item[0].replace(/\[[xX]\]/, "[ ]"), tail: "\n" };
};
