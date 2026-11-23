import { type Edit, lineEnd, lineStart, type Span } from "./pageEdits";
import { fenceLine } from "./pageLists";

type Fence = {
  readonly lead: string;
  readonly indent: string;
  readonly line: string;
  readonly mark: string;
};

export type Mend = { readonly first: Fence | null; readonly last: Fence | null };

const MARKER = /(?:[-*+]|\d{1,9}[.)])(?=[ \t]*$)/;

export const fenceAt = (text: string, at: number): Fence | null => {
  const found = fenceLine(text.slice(lineStart(text, at), lineEnd(text, at)).replace(/\r$/, ""));
  if (!found) return null;
  const { lead, mark, rest } = found;
  const indent = lead.replace(MARKER, (marker) => " ".repeat(marker.length));
  return { lead, indent, line: mark + rest, mark };
};

const lineBegins = (text: string, at: number) => at === 0 || text[at - 1] === "\n";

const prefixAt = (text: string, at: number) =>
  /^[ \t>]*/.exec(text.slice(lineStart(text, at), at))?.[0] ?? "";

const blankLine = (prefix: string) => `\n${prefix.trimEnd()}\n`;

const shutBelow = (text: string, at: number, { mark }: Fence) => {
  if (!/^\r?\n/.test(text.slice(at))) return false;
  const next = text.indexOf("\n", at) + 1;
  const below = fenceLine(text.slice(next, lineEnd(text, next)).replace(/\r$/, ""));
  return (
    below !== null &&
    below.mark[0] === mark[0] &&
    below.mark.length >= mark.length &&
    below.rest.trim() === ""
  );
};

export const shutOf = (text: string, span: Span, typed: string, { first, last }: Mend) => {
  if (!first || shutBelow(text, span.to, first)) return "";
  const bare = typed === "" && text.slice(lineStart(text, span.from), span.from) === first.indent;
  const opens = bare ? "" : `\n${first.indent}`;
  const prefix = prefixAt(text, span.to);
  const parted = last || /^(?:\r?\n|$)/.test(text.slice(span.to)) ? "" : blankLine(prefix) + prefix;
  return `${opens}${first.mark}${parted}`;
};

export const openOf = (text: string, edit: Edit, { last }: Mend) => {
  if (!last) return "";
  const fresh = edit.insert === "" ? lineBegins(text, edit.from) : edit.insert.endsWith("\n");
  const rest = lineBegins(text, edit.to) ? "" : last.indent;
  const parted = fresh ? "" : blankLine(prefixAt(text, edit.from));
  return `${parted}${last.lead}${last.line}\n${rest}`;
};

export const mended = (text: string, edit: Edit, mend: Mend): Edit => {
  const shut = { ...edit, insert: edit.insert + shutOf(text, edit, edit.insert, mend) };
  return { ...shut, insert: shut.insert + openOf(text, shut, mend) };
};

export const carriedOf = (text: string, span: Span, mend: Mend | null): string => {
  const said = text.slice(span.from, span.to);
  const first = mend?.first;
  const last = mend?.last;
  const rest = lineBegins(text, span.from) ? "" : (first?.indent ?? "");
  const opened = first ? `${first.lead}${first.line}\n${rest}` : "";
  const shut = last ? `${said.endsWith("\n") ? "" : "\n"}${last.indent}${last.mark}` : "";
  return opened + said + shut;
};

export const apart = (text: string, at: number, said: string) => {
  const before = lineBegins(text, at) ? "" : "\n\n";
  const after = /^(?:\r?\n|$)/.test(text.slice(at)) ? "" : "\n\n";
  return `${before}${said}${after}`;
};
