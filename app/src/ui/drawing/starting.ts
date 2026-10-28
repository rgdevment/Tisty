import { t, type Word } from "../../locales";

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

export const fenced = (source: string): string => `\`\`\`mermaid\n${source}\`\`\``;

// Counted as the core counts, since the diagram window finds its block by both.
export interface Fence {
  readonly at: number;
  readonly source: string;
}

// A fence may sit inside quotes and a list item: the core starts it after their marks.
const OPENING =
  /^((?: {0,3}> ?)*(?: {0,3}(?:[-*+]|\d{1,9}[.)])[ \t]+| *))(`{3,}|~{3,})[ \t]*([^\s`]*)/;
const QUOTED = /^(?: {0,3}> ?)*/;
const DIAGRAM = /^(mermaid|mmd)$/;

// The core leaves the quote marks and the item's indent out of what a block says.
const insideOf = (lead: string) => {
  const quotes = (lead.match(/>/g) ?? []).length;
  const indent = new RegExp(`^ {0,${lead.replace(QUOTED, "").length}}`);
  return (line: string) => {
    let rest = line;
    for (let one = 0; one < quotes; one++) rest = rest.replace(/^ {0,3}> ?/, "");
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
    while (at < lines.length && !closing.test(inside(lines[at]))) {
      inner.push(`${inside(lines[at])}\n`);
      next();
    }
    if (at < lines.length) next();
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
