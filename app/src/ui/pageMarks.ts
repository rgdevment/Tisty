import type { Edit, Span } from "./pageEdits";

export type Marks = readonly (readonly [string, string])[];

const MARK = /\*\*|__|\+\+|~~|==|\*|_|`|\[|\]/g;
const LINK_END = /^\]\([^)\s]*(?:\s+"[^"]*")?\)/;
const CLOSER = /\*\*|__|\+\+|~~|==(?:\{[a-z]+\})?|\*|_|`/y;
const PAIRED = ["**", "__", "++", "~~", "==", "*", "_", "`"];

export const openOf = (wait: { readonly marks: Marks }) =>
  wait.marks.map(([open]) => open).join("");

export const closeOf = (wait: { readonly marks: Marks }) =>
  wait.marks
    .map(([, close]) => close)
    .reverse()
    .join("");

// Marks right after the caret close the words before it only when the line opened them, never a next word's.
export const closedTokens = (text: string, at: number): string[] => {
  const line = text.slice(text.lastIndexOf("\n", at - 1) + 1, at);
  const seen = new Map<string, number>();
  const count = (kind: string) => seen.get(kind) ?? 0;
  for (const [token] of line.matchAll(MARK)) seen.set(token, count(token) + 1);
  const found: string[] = [];
  let end = at;
  for (;;) {
    const link = LINK_END.exec(text.slice(end))?.[0];
    if (link && count("[") > count("]")) {
      seen.set("]", count("]") + 1);
      found.push(link);
      end += link.length;
      continue;
    }
    const stars = /^(?:\*+|_+)/.exec(text.slice(end))?.[0];
    const star = stars?.[0] ?? "";
    const twice = star + star;
    const pick =
      stars && stars.length > 1 && count(twice) % 2 ? twice : count(star) % 2 ? star : "";
    CLOSER.lastIndex = end;
    const token = stars ? pick : CLOSER.exec(text)?.[0];
    const kind = token?.startsWith("==") ? "==" : token;
    if (!token || !kind || count(kind) % 2 === 0) return found;
    seen.set(kind, count(kind) + 1);
    found.push(token);
    end += token.length;
  }
};

export const closedRun = (text: string, at: number) => at + closedTokens(text, at).join("").length;

export const pastClose = (text: string, at: number): number | null => {
  if (!/\S/.test(text[at - 1] ?? "")) return null;
  const line = text.slice(text.lastIndexOf("\n", at - 1) + 1, at);
  if ((line.match(/`/g)?.length ?? 0) % 2) return null;
  const end = closedRun(text, at);
  return end > at ? end : null;
};

// Stars and underscores stack, so a run of three is bold and italic at once and two alone are only bold.
const starred = (text: string, one: Span, open: string) => {
  const mark = open[0] ?? "*";
  let before = 0;
  while (text[one.from - before - 1] === mark) before++;
  let after = 0;
  while (text[one.to + after] === mark) after++;
  const most = Math.min(before, after);
  return open.length === 1 ? most % 2 === 1 : most >= 2;
};

const starsBefore = (text: string, at: number, star: string) => {
  let many = 0;
  while (text[at - many - 1] === star) many++;
  return many;
};

// A format closed just before the caret is entered again rather than opened anew, so no marks stand back to back.
export const rejoined = (text: string, at: number, marks: Marks): [number, Marks] => {
  let from = at;
  let left = marks;
  for (;;) {
    const found = left.findIndex(([, close]) => {
      const star = /^(\*|_)\1?$/.test(close) ? (close[0] ?? "") : "";
      const run = star ? starsBefore(text, from, star) : 0;
      const shut = star
        ? close.length === 1
          ? run % 2 === 1
          : run >= 2
        : text.slice(from - close.length, from) === close;
      const edge = text[from - (star ? run : close.length) - 1] ?? "";
      return shut && /\S/.test(edge);
    });
    if (found < 0) return [from, left];
    from -= left[found]?.[1].length ?? 0;
    left = left.filter((_, index) => index !== found);
  }
};

export const unpaired = (text: string, one: Span) => {
  let { from, to } = one;
  if (from === to) return one;
  for (;;) {
    const mark = PAIRED.find(
      (one) => text.slice(from - one.length, from) === one && text.startsWith(one, to),
    );
    if (!mark) return { from, to };
    const close = mark === "==" ? (/^==(?:\{[a-z]+\})?/.exec(text.slice(to))?.[0] ?? mark) : mark;
    from -= mark.length;
    to += close.length;
  }
};

export const trimmed = (text: string, one: Span | null) => {
  if (!one) return null;
  let { from, to } = one;
  while (from < to && /\s/.test(text[from] ?? "")) from++;
  while (to > from && /\s/.test(text[to - 1] ?? "")) to--;
  return from < to ? { from, to } : null;
};

// Words partly in a format take it whole, so the marks inside them go before the new pair goes around.
const bareOf = (inner: string, open: string) => {
  if (open === "*") return inner.replace(/(?<!\*)\*(?!\*)/g, "");
  if (open === "_") return inner.replace(/(?<!_)_(?!_)/g, "");
  if (open.startsWith("==")) return inner.replace(/==(?:\{[a-z]+\})?/g, "");
  return inner.split(open).join("");
};

// A choice that starts or ends inside a format takes that format whole, so no mark is left without its partner.
const MARK_CHAR = /[*_~+=`]/;

export const covered = (text: string, cut: Span, open: string) => {
  const one = { ...cut };
  while (
    one.from > 0 &&
    MARK_CHAR.test(text[one.from - 1] ?? "") &&
    MARK_CHAR.test(text[one.from] ?? "")
  )
    one.from--;
  while (
    one.to < text.length &&
    MARK_CHAR.test(text[one.to - 1] ?? "") &&
    MARK_CHAR.test(text[one.to] ?? "")
  )
    one.to++;
  const mark = open.startsWith("==") ? "==" : open;
  const single = mark === "*" || mark === "_";
  const lone = single ? new RegExp(`(?<!\\${mark})\\${mark}(?!\\${mark})`, "g") : null;
  const count = (said: string) =>
    lone ? (said.match(lone)?.length ?? 0) : said.split(mark).length - 1;
  if (!mark || count(text.slice(one.from, one.to)) % 2 === 0) return one;
  const start = text.lastIndexOf("\n", one.from - 1) + 1;
  if (count(text.slice(start, one.from)) % 2 === 1) {
    const before = text.slice(start, one.from);
    const at = lone ? [...before.matchAll(lone)].pop()?.index : before.lastIndexOf(mark);
    return at === undefined || at < 0 ? one : { from: start + at, to: one.to };
  }
  const after = text.slice(one.to);
  const at = lone ? after.search(lone) : after.indexOf(mark);
  return at < 0 ? one : { from: one.from, to: one.to + at + mark.length };
};

// A format already around the words is taken off, as a second press of a button undoes the first.
export const worn = (text: string, one: Span, open: string, close: string): Edit => {
  const inner = text.slice(one.from, one.to);
  // Words chosen with their marks, as a double click past them takes, are words already worn.
  const single = open.length === 1 && inner.startsWith(open.repeat(2));
  if (!single && inner.length > open.length + close.length) {
    const within = inner.slice(open.length, inner.length - close.length);
    if (inner.startsWith(open) && inner.endsWith(close) && bareOf(within, open) === within)
      return { ...one, insert: within };
  }
  const around = /^(\*|_)\1?$/.test(open)
    ? starred(text, one, open)
    : text.slice(one.from - open.length, one.from) === open &&
      text.slice(one.to, one.to + close.length) === close;
  return around
    ? { from: one.from - open.length, to: one.to + close.length, insert: inner }
    : { ...one, insert: `${open}${bareOf(inner, open)}${close}` };
};

const LINE_MARKS =
  /^[ \t]*(?:>[ \t]?)*[ \t]*(?:#{1,6}[ \t]+|[-*+][ \t]+(?:\[[ xX]\][ \t]+)?|\d+[.)][ \t]+)?/;

// A mark never crosses a line, so words chosen over several blocks take a pair in each, past each line's own marks.
export const wornEach = (text: string, one: Span, open: string, close: string): Edit => {
  const lines = text.slice(one.from, one.to).split(/(\r?\n)/);
  const insert = lines
    .map((line, at) => {
      if (at % 2 === 1) return line;
      const lead = at === 0 ? "" : (LINE_MARKS.exec(line)?.[0] ?? "");
      const body = line.slice(lead.length);
      const words = body.trimEnd();
      return words.trim() ? `${lead}${open}${words}${close}${body.slice(words.length)}` : line;
    })
    .join("");
  return { from: one.from, to: one.to, insert };
};
