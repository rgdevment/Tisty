import { applied, changeOf, type Edit, lineEnd, lineStart, type Span } from "./pageEdits";
import { fenceWalker, LEAD, STANDS_ALONE } from "./pageLists";

const OPENS = /^:::[\p{L}\p{N}]/u;
const SHUTS = /^:::[ \t]*\r?$/;
const BLANK = /^[ \t]*\r?$/;
const BREAKS_BELOW =
  /^(?:#{1,6}(?:[ \t]|\r?$)|`{3,}|~{3,}|>|[-*+][ \t]|1[.)][ \t]|(?:[*_][ \t]*){3,}\r?$)/;
const TRAILING_BLANKS = /(?:\r?\n[ \t]*(?:>[ \t]?)*[ \t]*)+$/;
const LEADING_BLANKS = /^(?:\r?\n[ \t]*(?:>[ \t]?)*[ \t]*)+/;
const QUOTE = /^[ \t]*(?:>[ \t]?)*/;
const MARKER = /^([ \t]*(?:>[ \t]?)*[ \t]*):::([\p{L}\p{N}]\S*)[ \t]*/u;
const STAND_IN = "\u0000";

type Colon = { readonly open: Span; readonly shut: Span };

const lineIn = (text: string, at: number): Span => ({
  from: lineStart(text, at),
  to: lineEnd(text, at),
});

const bodyOf = (line: string) => line.slice(LEAD.exec(line)?.[0].length ?? 0);

const lineAbove = (text: string, at: number) =>
  at === 0 ? null : text.slice(lineStart(text, at - 1), at - 1);

const lineBelow = (text: string, at: number) =>
  at >= text.length ? null : text.slice(at + 1, lineEnd(text, at + 1));

export const colonLine = (line: string) => bodyOf(line).startsWith(":::");

const paragraphAlone = (text: string, line: Span) => {
  const above = lineAbove(text, line.from) ?? "";
  const below = bodyOf(lineBelow(text, line.to) ?? "");
  const opens = BLANK.test(bodyOf(above)) || STANDS_ALONE.test(above);
  return opens && (BLANK.test(below) || BREAKS_BELOW.test(below));
};

const nameIn = (text: string, open: Span): Span => {
  const said = text.slice(open.from, open.to).replace(/\r$/, "");
  const marked = MARKER.exec(said);
  const at = open.from + (marked?.[0].length ?? 0);
  const words = said.slice(at - open.from).trimEnd();
  if (words) return { from: at, to: at + words.length };
  const kind = open.from + (marked?.[1]?.length ?? 0) + 3;
  return { from: kind, to: kind + (marked?.[2]?.length ?? 0) };
};

const leadOf = (line: string) => LEAD.exec(line)?.[0] ?? "";

const colonsIn = (text: string): Colon[] => {
  const found: Colon[] = [];
  const opened: Span[] = [];
  const walked = fenceWalker();
  for (let from = 0; ; ) {
    const line = lineIn(text, from);
    const said = text.slice(line.from, line.to);
    const body = bodyOf(said);
    if (!walked(said).coded && paragraphAlone(text, line)) {
      if (SHUTS.test(body)) {
        const lead = leadOf(said);
        const inner = (open: Span) => !lead.startsWith(leadOf(text.slice(open.from, open.to)));
        while (opened.length > 0 && inner(opened[opened.length - 1] as Span)) opened.pop();
        const open = opened.pop();
        if (open) found.push({ open, shut: line });
      } else if (OPENS.test(body)) opened.push(line);
    }
    if (line.to >= text.length) return found;
    from = line.to + 1;
  }
};

export const colonPairs = (text: string): Span[] =>
  colonsIn(text).map(({ open, shut }) => ({ from: open.from, to: shut.to }));

const overlaps = (cut: Span, span: Span) => cut.from < span.to && cut.to > span.from;

const takes = (cut: Span, line: Span) =>
  overlaps(cut, line) && !(cut.from >= line.from && cut.to <= line.to);

const quoteAt = (text: string, at: number) =>
  QUOTE.exec(text.slice(lineStart(text, at), at))?.[0] ?? "";

const crossesColons = (text: string, cut: Span) => {
  const said = text.slice(cut.from, cut.to);
  if (!said.includes("\n")) return false;
  const first = lineIn(text, cut.from);
  return said.includes(":::") || colonLine(text.slice(first.from, first.to));
};

const apartIn = (text: string, line: Span) => {
  const quote = QUOTE.exec(text.slice(line.from, line.to))?.[0] ?? "";
  return `\n${quote.trimEnd()}\n`;
};

const apartAfter = (text: string, at: number, quote: string) => {
  const end = lineEnd(text, at);
  if (at < end) return `\n${quote.trimEnd()}\n${quote}`;
  return bodyOf(lineBelow(text, end) ?? "").trim() ? `\n${quote.trimEnd()}` : "";
};

const apartBefore = (text: string, at: number, quote: string) => {
  if (bodyOf(text.slice(lineStart(text, at), at))) return `\n${quote.trimEnd()}\n${quote}`;
  return bodyOf(lineAbove(text, lineStart(text, at)) ?? "").trim() ? `\n${quote}` : "";
};

const withoutLines = (text: string, from: number, lines: readonly Span[]) => {
  let left = "";
  let at = from;
  for (const line of lines) {
    left += text.slice(at, line.from).replace(TRAILING_BLANKS, "");
    at = line.to;
  }
  return { left, at };
};

const linesOf = (text: string, lines: readonly Span[]) =>
  lines.map((one) => text.slice(one.from, one.to).replace(/\r$/, ""));

type Crossing = {
  readonly from: number;
  readonly to: number;
  readonly dropped: readonly Span[];
  readonly kept: readonly Span[];
};

const crossingOf = (text: string, cut: Span): Crossing | null => {
  if (cut.from >= cut.to || !crossesColons(text, cut)) return null;
  let from = cut.from;
  let to = cut.to;
  const dropped: Span[] = [];
  const kept: Span[] = [];
  for (const { open, shut } of colonsIn(text)) {
    const name = nameIn(text, open);
    const named = takes(cut, open) && overlaps(cut, name);
    const closed = takes(cut, shut);
    const lead = open.from + (LEAD.exec(text.slice(open.from, open.to))?.[0].length ?? 0);
    if (named) from = Math.min(from, lead);
    if (named && cut.to < open.to) to = Math.max(to, open.to);
    if (named && !closed) dropped.push(shut);
    if (!named && cut.from < open.from && cut.to > open.from && cut.to <= name.from)
      to = cut.from + text.slice(cut.from, open.from).replace(TRAILING_BLANKS, "").length;
    if (!named && cut.from >= name.to && cut.from <= open.to && cut.to > open.to)
      from = open.to + (LEADING_BLANKS.exec(text.slice(open.to))?.[0].length ?? 0);
    if (!named && closed && cut.from >= open.to) kept.push(shut);
  }
  if (from === cut.from && to === cut.to && !dropped.length && !kept.length) return null;
  return { from, to, dropped: dropped.sort((a, b) => a.from - b.from), kept };
};

const keptAfter = (text: string, at: number, kept: readonly Span[]) => {
  const last = kept[kept.length - 1];
  if (!last) return "";
  const apart = apartIn(text, last);
  const quote = QUOTE.exec(text.slice(last.from, last.to))?.[0] ?? "";
  return `${apart}${linesOf(text, kept).join(apart)}${apartAfter(text, at, quote)}`;
};

export const colonsCrossed = (text: string, chosen: Span) => crossingOf(text, chosen) !== null;

export const colonsKept = (text: string, edit: Edit): Edit => {
  const crossing = crossingOf(text, edit);
  if (!crossing) return edit;
  const { left, at } = withoutLines(text, crossing.to, crossing.dropped);
  const insert = `${edit.insert}${keptAfter(text, crossing.to, crossing.kept)}${left}`;
  return { from: crossing.from, to: Math.max(at, crossing.to), insert };
};

export const colonsCarried = (text: string, chosen: Span): string => {
  const said = text.slice(chosen.from, chosen.to);
  const crossing = crossingOf(text, chosen);
  if (!crossing) return said;
  const { left, at } = withoutLines(text, crossing.from, crossing.kept);
  const taken = `${left}${text.slice(at, crossing.to)}`;
  const first = crossing.dropped[0];
  if (!first) return taken;
  const apart = apartIn(text, first);
  return `${taken}${apart}${linesOf(text, crossing.dropped).join(apart)}`;
};

const landed = (text: string, at: number, said: string) => {
  const quote = quoteAt(text, at);
  const opens = colonLine(said.slice(0, lineEnd(said, 0)));
  const shuts = colonLine(said.slice(lineStart(said, said.length)));
  const before = opens ? apartBefore(text, at, quote) : "";
  const after = shuts ? apartAfter(text, at, quote) : "";
  const quoted = said
    .split("\n")
    .map((line, at) => (at === 0 ? line : `${line.trim() ? quote : quote.trimEnd()}${line}`))
    .join("\n");
  return `${before}${quoted}${after}`;
};

export const colonsMoved = (text: string, words: Span, to: number): [Edit, number] | null => {
  if (to >= words.from && to <= words.to) return null;
  const said = colonsCarried(text, words);
  const marked = `${text.slice(0, to)}${STAND_IN}${text.slice(to)}`;
  const shift = (at: number) => (at >= to ? at + 1 : at);
  const gone = colonsKept(marked, { from: shift(words.from), to: shift(words.to), insert: "" });
  const left = applied(marked, gone);
  const at = left.indexOf(STAND_IN);
  if (at < 0) return null;
  const rest = `${left.slice(0, at)}${left.slice(at + 1)}`;
  const put = landed(rest, at, said);
  return [changeOf(text, `${rest.slice(0, at)}${put}${rest.slice(at)}`), at + put.length];
};
