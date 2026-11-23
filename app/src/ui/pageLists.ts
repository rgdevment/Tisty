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

export const LEAD = /^[ \t]*(?:>[ \t]?)*[ \t]*/;

export const hardBreak = (text: string, at: number) => {
  const { line } = lineAt(text, at);
  if (HEADED.test(line)) return parted(text, at, false);
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

function* linesBelow(text: string, start: number) {
  for (let from = text.indexOf("\n", start) + 1; from > 0; ) {
    const ends = text.indexOf("\n", from);
    yield { from, line: text.slice(from, ends < 0 ? text.length : ends).replace(/\r$/, "") };
    from = ends + 1;
  }
}

const columns = (space: string, from = 0) => {
  let at = from;
  for (const one of space) at = one === "\t" ? at + 4 - (at % 4) : at + 1;
  return at - from;
};

const quotedAs = (quotes: number) => new RegExp(`^(?:[ \\t]*>){${quotes}}(?:(?<=>)[ \\t])?`);

const kindOf = (line: string) => {
  const item = LISTED.exec(line);
  return item ? (item[4] ?? item[2]) : null;
};

const opensBlock = (words: string) =>
  isItem(words) || STANDS_ALONE.test(words) || words.startsWith(">");

const movedLeft = (line: string, quote: RegExp, by: number) => {
  const mark = quote.exec(line)?.[0] ?? "";
  let cut = mark.length;
  let at = 0;
  while (at < by && /[ \t]/.test(line[cut] ?? "")) {
    at += columns(line[cut], at);
    cut += 1;
  }
  return `${mark}${" ".repeat(Math.max(0, at - by))}${line.slice(cut)}`;
};

type Held = { readonly end: number; readonly text: string };

type Indented = { readonly quote: RegExp; readonly own: number; readonly content: number };

type Risen = { readonly from: number; readonly first: number; readonly apart: boolean };

const linesUnder = (text: string, ends: number, said: string, indented: Indented) => {
  const { quote, own, content } = indented;
  const walked = fenceWalker();
  let open = walked(said).open;
  let goesOn = !open && !STANDS_ALONE.test(said);
  let words = ends;
  let end = ends;
  let blank = -1;
  let risen: Risen | null = null;
  let top: string | null = null;
  for (const { from, line } of linesBelow(text, ends)) {
    const mark = quote.exec(line)?.[0];
    if (mark === undefined) break;
    const inner = line.slice(mark.length);
    const space = /^[ \t]*/.exec(inner)?.[0] ?? "";
    const rest = inner.slice(space.length);
    if (!rest) {
      if (blank < 0) blank = from;
      continue;
    }
    const at = columns(space);
    const deep = at >= content;
    const coded = open;
    open = walked(inner).open;
    const opens = !coded && at - (deep ? content : own) < 4 && opensBlock(rest);
    if (!deep && (blank >= 0 || opens || !goesOn)) break;
    if (!risen && (blank >= 0 || opens))
      risen = { from: blank >= 0 ? blank : from, first: at, apart: blank >= 0 };
    if (risen) end = from + line.length;
    else words = from + line.length;
    if (risen && deep && at <= risen.first && !coded) top = rest;
    blank = -1;
    goesOn = !coded && !open && !STANDS_ALONE.test(rest);
  }
  return { words, end, risen, top };
};

const heldUnder = (
  text: string,
  [start, line]: readonly [number, string],
  found: RegExpExecArray,
  it: Indent,
) => {
  const ends = start + line.length;
  const quote = quotedAs(it.quotes);
  const own = columns(found[1].slice(it.quote.length));
  const gap = columns(found[5], own + found[2].length);
  const content = own + found[2].length + (gap > 4 ? 1 : gap);
  const said = line.slice(found[0].length);
  const under = linesUnder(text, ends, said, { quote, own, content });
  const { risen, end } = under;
  if (!risen) return { end: under.words, text: text.slice(ends, under.words), joins: () => false };
  const shift = (risen.first - content >= 4 ? content : risen.first) - own;
  const lifted = text
    .slice(risen.from, end)
    .split("\n")
    .map((one) => movedLeft(one, quote, shift))
    .join("\n");
  const apart = risen.apart ? "" : `${found[1].trimEnd()}\n`;
  const kind = under.top === null ? null : kindOf(under.top);
  return {
    end,
    text: `${text.slice(ends, risen.from)}${apart}${lifted}`,
    joins: (next: string) => kind !== null && kindOf(next) === kind,
  };
};

const freed = (
  text: string,
  [start, line]: readonly [number, string],
  [lead, marks]: readonly [string, number],
  apartFrom: (next: string) => boolean,
  held: Held = { end: start + line.length, text: "" },
): Moved => {
  const ends = text.indexOf("\n", held.end);
  const next = ends < 0 ? null : lineAt(text, ends + 1).line;
  const above = start > 0 && !wordless(lineAt(text, start - 1).line) ? `${lead.trimEnd()}\n` : "";
  const below = next !== null && apartFrom(next) ? `\n${lead.trimEnd()}` : "";
  const words = start + line.length;
  const to = below || held.text !== text.slice(words, held.end) ? held.end : words;
  const insert = `${above}${lead}${line.slice(marks)}${to > words ? held.text : ""}${below}`;
  return {
    edit:
      to > words && ends >= 0
        ? { from: start, to: ends + 1, insert: `${insert}${text.slice(to, ends + 1)}` }
        : { from: start, to, insert },
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
  const it = indentOf(line);
  const held = it ? heldUnder(text, [start, line], found, it) : null;
  const listed = (next: string) => {
    const below = indentOf(next);
    return below !== null && below.lead - (it?.lead ?? 0) < 4 && !held?.joins(next);
  };
  return freed(text, [start, line], [found[1] ?? "", found[0].length], listed, held ?? undefined);
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
const FENCE_OPENED = /^((?:[ \t]*>)*)[ \t]*(?:(?:[-*+]|\d{1,9}[.)])[ \t]+)?(`{3,}|~{3,})(.*)$/;
const IN_QUOTE = /^[ \t]*>/;
export const STANDS_ALONE =
  /^[ \t]*(?:>[ \t]?)*[ \t]*(?:#{1,6}(?:[ \t]|$)|`{3,}|~{3,}|\||(?:-[ \t]*){3,}$|(?:\*[ \t]*){3,}$|(?:_[ \t]*){3,}$|=+[ \t]*$)/;

type Fence = { readonly mark: string; readonly quoted: boolean; readonly within: number };

// A fence closes only on a run of its own mark at least as long, or where the quote it opened in ends.
export const fenceWalker = () => {
  let open: { mark: string; quoted: boolean } | null = null;
  return (raw: string) => {
    const line = raw.replace(/\r$/, "");
    if (open?.quoted && !IN_QUOTE.test(line)) open = null;
    const before = open !== null;
    const found = FENCE.exec(line);
    if (found) {
      const [, quotes, mark, rest] = found;
      if (open === null) {
        if (mark[0] !== "`" || !rest.includes("`")) open = { mark, quoted: quotes !== "" };
      } else if (mark[0] === open.mark[0] && mark.length >= open.mark.length && !rest.trim())
        open = null;
    }
    return { coded: before || open !== null, open: open !== null };
  };
};

const inFence = (text: string, at: number) => {
  const walked = fenceWalker();
  let inside = false;
  for (const raw of text.slice(0, at).split("\n")) inside = walked(raw).open;
  return inside;
};

export const headingAt = (text: string, at: number) => {
  const { line } = lineAt(text, at);
  const item = LISTED.exec(line);
  return HEADED.test(item ? line.slice(item[0].length) : line) && !inFence(text, at);
};

export const fenceLine = (line: string) => {
  const found = FENCE_OPENED.exec(line);
  if (!found) return null;
  const [, quotes, mark, rest] = found;
  return { quotes, lead: line.slice(0, line.length - rest.length - mark.length), mark, rest };
};

const widthOf = (said: string) => said.replace(/\t/g, "    ").length;

const shuts = (was: Fence, line: string) => {
  const found = FENCE.exec(line);
  if (!found) return false;
  const [, , mark, rest] = found;
  return mark[0] === was.mark[0] && mark.length >= was.mark.length && !rest.trim();
};

const leftBy = (was: Fence, line: string) =>
  (was.quoted && !IN_QUOTE.test(line)) ||
  (!/^[ \t>]*$/.test(line) && widthOf(/^[ \t>]*/.exec(line)?.[0] ?? "") < was.within);

// A fence closes only on a run of its own mark at least as long, or where the quote or the item it opened in ends.
const fenceAfter = (was: Fence | null, raw: string): Fence | null => {
  const line = raw.replace(/\r$/, "");
  if (was && shuts(was, line)) return null;
  if (was && !leftBy(was, line)) return was;
  const opened = fenceLine(line);
  if (!opened || (opened.mark[0] === "`" && opened.rest.includes("`"))) return null;
  const within = LISTED.test(line) ? widthOf(opened.lead) : 0;
  return { mark: opened.mark, quoted: opened.quotes !== "", within };
};

const fencesBlanked = (text: string, openToEnd: boolean): string => {
  const lines = text.split("\n");
  const blanked = (from: number, to: number) => {
    for (let at = from; at < to; at++) lines[at] = " ".repeat(lines[at].length);
  };
  let open: Fence | null = null;
  let since = 0;
  for (const [at, line] of lines.entries()) {
    const was: Fence | null = open;
    open = fenceAfter(was, line);
    const shut = !open && FENCE.test(line.replace(/\r$/, ""));
    if (was && open !== was) blanked(since, shut ? at + 1 : at);
    if (open && open !== was) since = at;
  }
  if (open && openToEnd) blanked(since, lines.length);
  return lines.join("\n");
};

export const fencedOut = (text: string): string => fencesBlanked(text, false);

export const codeOut = (text: string): string => fencesBlanked(text, true);

const paragraphLine = (line: string) =>
  !LISTED.test(line) && !wordless(line) && !STANDS_ALONE.test(line);

const quotesOf = (line: string) => (/^[ \t>]*/.exec(line)?.[0].split(">").length ?? 1) - 1;

const opensItem = (line: string) => {
  const item = LISTED.exec(line);
  const first = item?.[3] === undefined || Number(item[3]) === 1;
  return item !== null && first && line.slice(item[0].length).trim() !== "";
};

export const linesJoin = (above: string, line: string) => {
  const [was, now] = [above, line].map((one) => one.replace(/\r$/, ""));
  return (
    !opensItem(now) &&
    !wordless(now) &&
    !STANDS_ALONE.test(now) &&
    !wordless(was) &&
    !STANDS_ALONE.test(was)
  );
};

const siblingItems = (opening: string, line: string) => {
  const [was, now] = [LISTED.exec(opening), LISTED.exec(line)];
  if (!was || !now) return false;
  const kind = (item: RegExpExecArray) => (item[3] === undefined ? "bullet" : "number");
  return widthOf(was[1]) === widthOf(now[1]) && kind(was) === kind(now);
};

export const carriesOn = (opening: string, above: string, line: string) =>
  linesJoin(above, line) && quotesOf(line) <= quotesOf(opening) && !siblingItems(opening, line);

const RULED =
  /^[ \t]*(?:>[ \t]?)*[ \t]*(?:=+|-+|(?:-[ \t]*){3,}|(?:\*[ \t]*){3,}|(?:_[ \t]*){3,})[ \t]*$/;
const DELIMITER_ROW =
  /^[ \t]*(?:>[ \t]?)*[ \t]*\|?(?:[ \t]*:?-+:?[ \t]*\|)*[ \t]*:?-+:?[ \t]*\|?[ \t]*$/;
const PIPED = /^[ \t]*(?:>[ \t]?)*[ \t]*\|/;

const headsTable = (head: string, line: string) =>
  head.includes("|") && !wordless(head) && line.includes("|") && DELIMITER_ROW.test(line);

export const tablesAndRulesOut = (text: string): string => {
  const lines = text.split("\n");
  const said = lines.map((one) => one.replace(/\r$/, ""));
  let table = false;
  for (const [at, line] of said.entries()) {
    table =
      headsTable(line, said[at + 1] ?? "") || (table && (PIPED.test(line) || paragraphLine(line)));
    if (table || RULED.test(line)) lines[at] = " ".repeat(lines[at].length);
  }
  return lines.join("\n");
};

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

const tightened = (
  text: string,
  [asked, at]: [Edit, number],
  left: string,
  landed: number,
): [Edit, number] => {
  const { line } = lineAt(left, landed);
  const own = indentOf(line);
  const ends = left.indexOf("\n", landed);
  const after = ends < 0 ? -1 : left.indexOf("\n", ends + 1);
  const next = after < 0 ? null : indentOf(lineAt(left, after + 1).line);
  const blank = ends >= 0 && wordless(lineAt(left, ends + 1).line);
  const same = own && next && next.quote === own.quote && next.lead === own.lead;
  if (!blank || !same || !paragraphLine(lineAt(text, asked.to).line) || inFence(left, ends))
    return [asked, at];
  const shift = asked.to - landed;
  const gap = ends + 1 + shift;
  return [
    {
      from: asked.from,
      to: after + 1 + shift,
      insert: `${asked.insert}${text.slice(asked.to, gap)}`,
    },
    at,
  ];
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
  if (isItem(line)) return tightened(text, kept, left, landed);
  const ends = left.indexOf("\n", landed);
  const item = ends < 0 ? null : LISTED.exec(lineAt(left, ends + 1).line);
  const number = item?.[3];
  const deep = columns(item?.[1].replace(/^.*>/, "") ?? "") >= 4;
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
