import { t, type Word } from "../../locales";
import { type DiagramKind, headerAt } from "./framing";

export type Start = "mindmap" | "erDiagram" | "flowchart" | "classDiagram" | "other";

export const STARTS: readonly Start[] = [
  "mindmap",
  "erDiagram",
  "flowchart",
  "classDiagram",
  "other",
];

const WORDS: Record<Start, { name: Word; source: Word }> = {
  mindmap: { name: "diagramMindmap", source: "diagramStartMindmap" },
  erDiagram: { name: "diagramSchema", source: "diagramStartSchema" },
  flowchart: { name: "diagramFlow", source: "diagramStartFlow" },
  classDiagram: { name: "diagramClasses", source: "diagramStartClasses" },
  other: { name: "diagramKindOther", source: "diagramStartOther" },
};

export const nameOf = (kind: Start): string => t(WORDS[kind].name);

// An empty block would draw nothing to edit.
export const startOf = (kind: Start): string => t(WORDS[kind].source);

// A copied diagram may hold backticks of its own, so its fence is longer than any run inside.
export const fenced = (source: string): string => {
  const longest = Math.max(2, ...(source.match(/`+/g) ?? []).map((run) => run.length));
  const fence = "`".repeat(longest + 1);
  return `${fence}mermaid\n${source}${fence}`;
};

const KIND_WORDS: Record<DiagramKind, Word> = {
  mindmap: "diagramMindmap",
  erDiagram: "diagramSchema",
  flowchart: "diagramFlow",
  classDiagram: "diagramClasses",
  other: "diagramOther",
};

export const kindName = (kind: DiagramKind): string => t(KIND_WORDS[kind]);

// The line after the header tells two diagrams of one kind apart in a list.
export const firstWords = (source: string): string => {
  const lines = source.split("\n");
  const header = headerAt(lines, /^\s*[^\s%]/);
  return (
    lines
      .slice(header + 1)
      .find((line) => line.trim() && !/^\s*%%/.test(line))
      ?.trim() ?? ""
  );
};

// Counted as the core counts, since the diagram window finds its block by both.
export interface Fence {
  readonly at: number;
  readonly source: string;
}

// The core starts a fence after quote and list marks, and four spaces make indented code instead.
const OPENING =
  /^((?: {0,3}> ?)*(?: {0,3}(?:[-*+]|\d{1,9}[.)])[ \t]+| {0,3}))(`{3,}|~{3,})[ \t]*([^\s`]*)/;
const QUOTED = /^(?: {0,3}> ?)*/;
const MARKED = /^ {0,3}(?:[-*+]|\d{1,9}[.)])/;
const DIAGRAM = /^(mermaid|mmd)$/;

// The core drops quote marks and item indent, and ends an unclosed block where they end (null).
const insideOf = (lead: string) => {
  const quotes = (lead.match(/>/g) ?? []).length;
  const own = lead.replace(QUOTED, "");
  const width = own.length;
  const indent = new RegExp(`^ {0,${width}}`);
  const item = MARKED.test(own);
  return (line: string): string | null => {
    let rest = line;
    for (let one = 0; one < quotes; one++) {
      if (!/^ {0,3}>/.test(rest)) return null;
      rest = rest.replace(/^ {0,3}> ?/, "");
    }
    if (item && rest.trim() && !rest.startsWith(" ".repeat(width))) return null;
    return rest.replace(indent, "");
  };
};

export const fencesOf = (text: string): Fence[] => {
  const lines = text.replace(/\r/g, "").replace(/\n$/, "").split("\n");
  const sized = new TextEncoder();
  const found: Fence[] = [];
  let bytes = 0;
  let at = 0;
  const next = () => {
    bytes += sized.encode(lines[at]).length + 1;
    at++;
  };
  while (at < lines.length) {
    const open = OPENING.exec(lines[at]);
    const start = bytes + sized.encode(open?.[1] ?? "").length;
    next();
    if (!open) continue;
    const mark = open[2];
    const inside = insideOf(open[1]);
    const closing = new RegExp(`^ {0,3}${mark[0]}{${mark.length},}\\s*$`);
    const inner: string[] = [];
    const lineNow = () => (at < lines.length ? inside(lines[at]) : null);
    let rest = lineNow();
    while (rest !== null && !closing.test(rest)) {
      inner.push(`${rest}\n`);
      next();
      rest = lineNow();
    }
    if (rest !== null) next();
    if (DIAGRAM.test(open[3])) found.push({ at: start, source: inner.join("") });
  }
  return found;
};

// The block the text before did not have at its place is the one just written, even beside a twin.
export const freshFence = (
  before: string,
  after: string,
  source: string,
): { nth: number; fence: Fence } | null => {
  const was = fencesOf(before);
  const now = fencesOf(after);
  const nth = now.findIndex((one, at) => one.source === source && was[at]?.source !== source);
  return nth < 0 ? null : { nth, fence: now[nth] };
};
