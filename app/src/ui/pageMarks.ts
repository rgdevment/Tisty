import { type Edit, lineEnd, lineStart, pastFeeds, type Span } from "./pageEdits";

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

const ESCAPABLE = /[!-/:-@[-`{-~]/;

export const escapeTaken = (text: string, one: Span): Span => {
  if (one.from >= one.to || !ESCAPABLE.test(text[one.from] ?? "")) return one;
  let slashes = 0;
  while (text[one.from - 1 - slashes] === "\\") slashes++;
  return slashes % 2 === 1 ? { from: one.from - 1, to: one.to } : one;
};

type Mark = { readonly from: number; readonly to: number; readonly mate: Span };
type Marking = (one: Span) => Mark[];
type Reach = (at: number) => Span;
type Laid = { readonly edit: Edit; readonly caret: number };

const RUNS =
  /\\[!-/:-@[-`{-~]|<[A-Za-z][A-Za-z\d+.-]{1,31}:[^\s<>]*>|\$[^\s$](?:[^$\n]*[^\s$])?\$|`+|\*+|_+|~+|=+|\++|!?\[|\]/g;
const MARKISH = /[*_~=+`[]/;
const LINKED = /\]\([^)\s]*(?:\s+"[^"]*")?\)|\]\[[^\]]*\]/y;
const PEN = /\{[a-z]+\}/y;
const SPACE = /\s/;
const PUNCT = /[\p{P}\p{S}]/u;
const WORDY = /[\p{L}\p{N}]/u;

const flanking = (before: string, after: string) => {
  const outer = (one: string) => SPACE.test(one) || PUNCT.test(one);
  const left = !SPACE.test(after) && (!PUNCT.test(after) || outer(before));
  const right = !SPACE.test(before) && (!PUNCT.test(before) || outer(after));
  return [left, right] as const;
};

const sidesOf = (run: string, before: string, after: string): readonly [boolean, boolean] => {
  if (run === "++")
    return [!WORDY.test(before) && !SPACE.test(after), !WORDY.test(after) && !SPACE.test(before)];
  if (run === "==") return [!SPACE.test(after), !SPACE.test(before)];
  const [left, right] = flanking(before, after);
  if (run[0] !== "_") return [left, right];
  return [left && (!right || PUNCT.test(before)), right && (!left || PUNCT.test(after))];
};

type Opener = {
  readonly mark: string;
  readonly from: number;
  to: number;
  readonly size: number;
  readonly both: boolean;
};

const blocked = (open: Opener, size: number, both: boolean) =>
  (both || open.both) && (open.size + size) % 3 === 0 && (open.size % 3 !== 0 || size % 3 !== 0);

const emphasis = (
  stack: Opener[],
  [mark, run]: readonly [string, Span],
  [opens, closes]: readonly [boolean, boolean],
  pair: (open: Span, close: Span) => void,
) => {
  const size = run.to - run.from;
  let at = run.from;
  for (let nth = closes ? stack.length - 1 : -1; nth >= 0 && at < run.to; ) {
    const open = stack[nth];
    if (!open) break;
    if (open.mark !== mark || blocked(open, size, opens)) {
      nth--;
      continue;
    }
    stack.length = nth + 1;
    const many = open.to - open.from >= 2 && run.to - at >= 2 ? 2 : 1;
    pair({ from: open.to - many, to: open.to }, { from: at, to: at + many });
    open.to -= many;
    at += many;
    if (open.to > open.from) continue;
    stack.length = nth;
    nth--;
  }
  if (opens && at < run.to) stack.push({ mark, from: at, to: run.to, size, both: closes });
};

const ticksAfter = (line: string, from: number, many: number) => {
  for (let at = line.indexOf("`", from); at >= 0; ) {
    let end = at;
    while (line[end] === "`") end++;
    if (end - at === many) return at;
    at = line.indexOf("`", end);
  }
  return -1;
};

type Token = { readonly from: number; readonly to: number; mate: Token | null };

const marksOn = (text: string, start: number, end: number): Mark[] => {
  const line = text.slice(start, end);
  if (!MARKISH.test(line)) return [];
  const tokens: Token[] = [];
  const token = (from: number, to: number) => {
    const one: Token = { from: start + from, to: start + to, mate: null };
    tokens.push(one);
    return one;
  };
  const paired = (open: Token, close: Token) => {
    open.mate = close;
    close.mate = open;
  };
  const waiting = new Map<string, Token[]>();
  const emphases: Opener[] = [];
  const brackets: Token[] = [];
  RUNS.lastIndex = LINE_MARKS.exec(line)?.[0].length ?? 0;
  for (let found = RUNS.exec(line); found; found = RUNS.exec(line)) {
    const [run] = found;
    const at = found.index;
    const past = at + run.length;
    if (run[0] === "\\" || run[0] === "<" || run[0] === "$") continue;
    if (run[0] === "`") {
      const shut = ticksAfter(line, past, run.length);
      if (shut < 0) continue;
      paired(token(at, past), token(shut, shut + run.length));
      RUNS.lastIndex = shut + run.length;
    } else if (run.endsWith("[")) brackets.push(token(at, past));
    else if (run === "]") {
      LINKED.lastIndex = at;
      const link = LINKED.exec(line)?.[0];
      const open = brackets.pop();
      if (!link || !open) continue;
      paired(open, token(at, at + link.length));
      RUNS.lastIndex = at + link.length;
    } else if (run[0] === "*" || run[0] === "_") {
      const sides = sidesOf(run, line[at - 1] ?? " ", line[past] ?? " ");
      emphasis(emphases, [run[0], { from: at, to: past }], sides, (open, close) =>
        paired(token(open.from, open.to), token(close.from, close.to)),
      );
    } else if (run.length === 2) {
      const [opens, closes] = sidesOf(run, line[at - 1] ?? " ", line[past] ?? " ");
      const stack = waiting.get(run) ?? [];
      waiting.set(run, stack);
      const open = closes ? stack.pop() : undefined;
      PEN.lastIndex = past;
      const pen = run === "==" && open ? (PEN.exec(line)?.[0].length ?? 0) : 0;
      if (open) paired(open, token(at, past + pen));
      else if (opens) stack.push(token(at, past));
    }
  }
  return tokens
    .filter((one): one is Token & { mate: Token } => one.mate !== null)
    .sort((one, other) => one.from - other.from);
};

const QUIET = /^[^*_~=+`[\](){}]*$/;

const markingOf = (text: string): Marking => {
  const lines: { start: number; end: number; known: Mark[] }[] = [];
  const lineAt = (at: number) => {
    const held = lines.find((one) => one.start <= at && at <= one.end);
    if (held) return held;
    const start = lineStart(text, at);
    const end = lineEnd(text, at);
    const line = { start, end, known: marksOn(text, start, end) };
    lines.push(line);
    return line;
  };
  return (one) => {
    if (QUIET.test(text.slice(Math.max(0, one.from - 1), one.to + 1))) return [];
    const first = lineAt(one.from);
    return first.end < one.to ? [...first.known, ...lineAt(one.to).known] : first.known;
  };
};

const leftAlone = (marks: Mark[], one: Span) => {
  const within = (mark: Span) => mark.from >= one.from && mark.to <= one.to;
  return marks.filter(
    (mark) => mark.from < one.to && mark.to > one.from && !(within(mark) && within(mark.mate)),
  );
};

const reachKept = (text: string, one: Span, back: boolean, reach: Reach, marking: Marking) => {
  let span = one;
  for (;;) {
    const orphans = leftAlone(marking(span), span);
    const near = orphans[back ? orphans.length - 1 : 0];
    if (!near) return span;
    if (back && near.to < span.to) span = { from: near.to, to: span.to };
    else if (!back && near.from > span.from) span = { from: span.from, to: near.from };
    else if (near.mate.to === near.from || near.to === near.mate.from)
      return { from: Math.min(near.from, near.mate.from), to: Math.max(near.to, near.mate.to) };
    else {
      const at = back ? near.from : near.to;
      const left = back ? { from: span.from, to: at } : { from: at, to: span.to };
      if (left.from < left.to) span = left;
      else {
        span = reach(at);
        const edge = back ? wordsFrom(text, at) : lineEnd(text, at);
        if (back ? span.from < edge : span.to > edge) return { from: at, to: at };
      }
    }
  }
};

const said = (text: string, marks: Mark[]) =>
  marks.map((mark) => text.slice(mark.from, mark.to)).join("");

const LINKS = /^!?\[|^\]/;

const emptiedOrMet = (text: string, marks: Mark[], kept: Mark[], one: Span) => {
  let { from, to } = one;
  const closers = kept.filter((mark) => mark.mate.to <= from);
  const openers = kept.filter((mark) => mark.mate.from >= to);
  while (closers[0] && closers[0].mate.to === from) {
    from = closers[0].mate.from;
    closers.shift();
  }
  while (openers[openers.length - 1]?.mate.from === to) {
    to = openers[openers.length - 1]?.mate.to ?? to;
    openers.pop();
  }
  const around = () =>
    one.from < one.to && !closers.length && !openers.length
      ? marks.find(
          (mark) =>
            mark.to === from &&
            mark.mate.from === to &&
            !LINKS.test(text.slice(mark.from, mark.to)),
        )
      : undefined;
  for (let format = around(); format; format = around()) {
    from = format.from;
    to = format.mate.to;
  }
  const shutAt = (at: number) => marks.find((mark) => mark.to === at && mark.mate.to <= mark.from);
  const openAt = (at: number) =>
    marks.find((mark) => mark.from === at && mark.mate.from >= mark.to);
  for (;;) {
    const shut = closers.length ? closers[closers.length - 1] : shutAt(from);
    const open = openers.length ? openers[0] : openAt(to);
    if (!shut || !open || said(text, [shut]) !== said(text, [open])) break;
    if (closers.length) closers.pop();
    else from = shut.from;
    if (openers.length) openers.shift();
    else to = open.to;
  }
  return { from, to, closers, openers };
};

const FLANKED = /^[*_~=+]/;

const openedPast = (text: string, at: number, openers: Mark[]) => {
  const marks = said(text, openers);
  const flanked = openers.every((mark) => FLANKED.test(text.slice(mark.from, mark.to)));
  const spaces = flanked && marks ? (/^[ \t]+/.exec(text.slice(at))?.[0] ?? "") : "";
  return { to: at + spaces.length, insert: spaces + marks };
};

export const replaced = (
  text: string,
  one: Span,
  insert: string,
  marking = markingOf(text),
): Laid => {
  let { from, to } = one;
  const marks = from < to ? marking(one) : [];
  const orphans = leftAlone(marks, one);
  for (const mark of orphans) {
    if (mark.from < from) from = Math.min(mark.to, to);
    if (mark.to > to) to = Math.max(mark.from, from);
  }
  const kept = orphans.filter((mark) => mark.from >= from && mark.to <= to);
  if (!insert) {
    const left = emptiedOrMet(text, marks, kept, { from, to });
    const opened = openedPast(text, left.to, left.openers);
    const marksLeft = said(text, left.closers) + opened.insert;
    return { edit: { from: left.from, to: opened.to, insert: marksLeft }, caret: left.from };
  }
  const closers = kept.filter((mark) => mark.mate.to <= from);
  const openers = kept.filter((mark) => mark.mate.from >= to);
  while (openers[openers.length - 1]?.mate.from === to) to = openers.pop()?.mate.to ?? to;
  while (closers.length && said(text, closers.slice(-1)) === said(text, openers.slice(0, 1))) {
    closers.pop();
    openers.shift();
  }
  const opened = openedPast(text, to, openers);
  const broken = insert.search(/\r?\n/);
  const head = broken < 0 ? insert : insert.slice(0, broken);
  const shut = said(text, closers);
  const placed = `${head}${shut}${insert.slice(head.length)}${opened.insert}`;
  const caret = from + insert.length + (broken < 0 ? 0 : shut.length);
  return { edit: { from, to: opened.to, insert: placed }, caret };
};

type KeyReach = { readonly back: boolean; readonly reach: Reach };

const escapedWhole = (text: string, one: Span): Span => {
  const start = escapeTaken(text, one);
  let slashes = 0;
  while (text[start.to - 1 - slashes] === "\\") slashes++;
  const cut = slashes % 2 === 1 && ESCAPABLE.test(text[start.to] ?? "");
  return cut ? { from: start.from, to: start.to + 1 } : start;
};

export const keptGone = (text: string, one: Span, key: KeyReach | null): Edit => {
  const marking = markingOf(text);
  const reach = (at: number) => escapedWhole(text, key?.reach(at) ?? { from: at, to: at });
  const span = key ? reachKept(text, escapedWhole(text, one), key.back, reach, marking) : one;
  return replaced(text, span, "", marking).edit;
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
  /^[ \t]*(?:>[ \t]?)*[ \t]*(?:#{1,6}[ \t]+|[-*+][ \t]+(?:\[[ xX]\][ \t]+)?|\d+[.)][ \t]+(?:\[[ xX]\][ \t]+)?|\[\^[^\]\s]+\]:[ \t]*)?/;

const CLOSINGS = /^(?:[*_~+=`>]|\]\([^)\n]*\)|\]\[[^\]\n]*\])*\r?$/;

export const closingOnly = (said: string) => CLOSINGS.test(said);

export const wordsFrom = (text: string, at: number) => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const ends = text.indexOf("\n", start);
  const line = text.slice(start, ends < 0 ? text.length : ends);
  return Math.min(at, start + (LINE_MARKS.exec(line)?.[0].length ?? 0));
};

const MARK_ENDS = /[\s>:*_~+=`[]/;
const OPENINGS = /^(?:[*_~+=`]|!?\[)*$/;
const BLANKISH = /^[ \t>]*\r?$/;

const lineAt = (text: string, at: number) => text.slice(lineStart(text, at), lineEnd(text, at));

const wordsReached = (text: string, start: number, at: number) => {
  if (at > start && !MARK_ENDS.test(text[at - 1] ?? "")) return null;
  const lead = text.slice(start, at);
  const marks = LINE_MARKS.exec(lead)?.[0].length ?? 0;
  return OPENINGS.test(lead.slice(marks)) ? start + marks : null;
};

const pastGap = (text: string, at: number, back: boolean) => {
  let edge = pastFeeds(text, at, back);
  while (back ? edge > 0 : edge < text.length) {
    if (!BLANKISH.test(lineAt(text, edge))) return edge;
    edge = pastFeeds(text, back ? lineStart(text, edge) : lineEnd(text, edge), back);
  }
  return edge;
};

export const joinedAcross = (text: string, at: number, back: boolean): Span | null => {
  if (back) {
    const start = lineStart(text, at);
    const to = start === 0 ? null : wordsReached(text, start, at);
    return to === null ? null : { from: pastGap(text, start, true), to };
  }
  const end = lineEnd(text, at);
  if (end >= text.length || !closingOnly(text.slice(at, end))) return null;
  if (at === lineStart(text, at)) return { from: at, to: pastFeeds(text, end, false) };
  const from = text[end - 1] === "\r" ? end - 1 : end;
  return { from, to: wordsFrom(text, lineEnd(text, pastGap(text, end, false))) };
};

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

// Words cut or copied carry every format they start or end inside, so no mark is left without its partner.
export const balanced = (text: string, chosen: Span): Span => {
  let one = { ...chosen };
  for (const mark of PAIRED) one = covered(text, one, mark);
  for (const mark of PAIRED) {
    const before = text.slice(one.from - mark.length, one.from);
    if (before === mark && text.slice(one.to, one.to + mark.length) === mark)
      one = { from: one.from - mark.length, to: one.to + mark.length };
  }
  const pen =
    text.slice(one.to - 2, one.to) === "==" ? /^\{[a-z]+\}/.exec(text.slice(one.to)) : null;
  return pen ? { from: one.from, to: one.to + pen[0].length } : one;
};
