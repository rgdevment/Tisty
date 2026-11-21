export type Span = { from: number; to: number };
export type Edit = { readonly from: number; readonly to: number; readonly insert: string };

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

const leading = (unit: number) => unit >= 0xd800 && unit < 0xdc00;
const trailing = (unit: number) => unit >= 0xdc00 && unit < 0xe000;

// The core counts in bytes, so an edit that cuts a character in two takes in the whole character.
export const inWholeChars = (was: string, one: Edit): Edit => {
  const before = one.from > 0 && leading(was.charCodeAt(one.from - 1)) ? 1 : 0;
  const after = one.to < was.length && trailing(was.charCodeAt(one.to)) ? 1 : 0;
  if (!before && !after) return one;
  const from = one.from - before;
  const to = one.to + after;
  return { from, to, insert: `${was.slice(from, one.from)}${one.insert}${was.slice(one.to, to)}` };
};

export const unsplitPairs = (text: string, one: Edit): Edit => {
  const splits = (at: number) =>
    at > 0 && at < text.length && leading(text.charCodeAt(at - 1)) && trailing(text.charCodeAt(at));
  const from = splits(one.from) ? one.from - 1 : one.from;
  const to = one.to === one.from ? from : splits(one.to) ? one.to + 1 : one.to;
  return from === one.from && to === one.to ? one : { ...one, from, to };
};

export const landedAfter = (wanted: Edit, asked: Edit, at: number) => {
  if (at < wanted.from) return at;
  const before = wanted.from - asked.from;
  if (at <= wanted.from + wanted.insert.length) return at - before;
  return at - before - (asked.to - wanted.to);
};

const segmenter = (granularity: "grapheme" | "word") =>
  typeof Intl !== "undefined" && "Segmenter" in Intl
    ? new Intl.Segmenter(undefined, { granularity })
    : null;

const GRAPHEMES = segmenter("grapheme");
const WORDS = segmenter("word");

const lineStart = (text: string, at: number) => text.lastIndexOf("\n", at - 1) + 1;

const lineEnd = (text: string, at: number) => {
  const end = text.indexOf("\n", at);
  return end < 0 ? text.length : end;
};

const charBefore = (text: string, at: number) => {
  if (at <= 0) return 0;
  const start = lineStart(text, at);
  if (start === at) return text[at - 2] === "\r" ? at - 2 : at - 1;
  if (!GRAPHEMES) return at - (trailing(text.charCodeAt(at - 1)) && at - 2 >= start ? 2 : 1);
  let from = start;
  for (const one of GRAPHEMES.segment(text.slice(start, at))) from = start + one.index;
  return from;
};

const charAfter = (text: string, at: number) => {
  if (at >= text.length) return text.length;
  const end = lineEnd(text, at);
  if (end === at || (text[at] === "\r" && end === at + 1)) return end + 1;
  if (!GRAPHEMES) return at + (leading(text.charCodeAt(at)) ? 2 : 1);
  const [first] = GRAPHEMES.segment(text.slice(at, end));
  return at + (first?.segment.length ?? 1);
};

const CLOSE_WRITTEN =
  /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}\p{Script=Thai}\p{Script=Lao}\p{Script=Khmer}\p{Script=Myanmar}]/u;

const STOPS = /^\p{P}+$/u;

const wordBefore = (text: string, at: number) => {
  const run = /\S*\s*$/.exec(text.slice(lineStart(text, at), at))?.[0] ?? "";
  const chunk = run.trimEnd();
  if (!WORDS || !CLOSE_WRITTEN.test(chunk)) return at - run.length;
  const pieces = WORDS.segment(chunk);
  let last = pieces.containing(chunk.length - 1);
  while (last && last.index > 0 && STOPS.test(last.segment))
    last = pieces.containing(last.index - 1);
  return at - run.length + (last?.index ?? 0);
};

const wordAfter = (text: string, at: number) => {
  const run = /^\s*\S*/.exec(text.slice(at, lineEnd(text, at)))?.[0] ?? "";
  const chunk = run.trimStart();
  if (!WORDS || !CLOSE_WRITTEN.test(chunk)) return at + run.length;
  let reach = chunk.length;
  for (const piece of WORDS.segment(chunk)) {
    reach = piece.index + piece.segment.length;
    if (!STOPS.test(piece.segment)) break;
  }
  return at + run.length - chunk.length + reach;
};

export const reachIn = (text: string, at: number, kind: string): Span => {
  const back = kind.endsWith("Backward");
  if (kind.includes("Line")) {
    if (back) {
      const start = lineStart(text, at);
      return { from: start < at ? start : charBefore(text, at), to: at };
    }
    const end = lineEnd(text, at);
    return { from: at, to: end > at ? end : charAfter(text, at) };
  }
  if (kind.includes("Word")) {
    if (back) {
      const from = wordBefore(text, at);
      return { from: from < at ? from : charBefore(text, at), to: at };
    }
    const to = wordAfter(text, at);
    return { from: at, to: to > at ? to : charAfter(text, at) };
  }
  return back ? { from: charBefore(text, at), to: at } : { from: at, to: charAfter(text, at) };
};
