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

/** The short example a new diagram begins with: an empty block would draw nothing to edit. */
export const startOf = (kind: Start): string => t(WORDS[kind].source);

export const fenced = (source: string): string => `\`\`\`mermaid\n${source}\`\`\``;

export interface Fence {
  /** Where its opening fence starts, in bytes, past any indent, as the core counts. */
  readonly at: number;
  /** Its lines, each ending in a line break, as the core reads them. */
  readonly source: string;
}

const OPENING = /^( {0,3})(`{3,}|~{3,})[ \t]*([^\s`]*)/;
const DIAGRAM = /^(mermaid|mmd)$/;

/** The diagram blocks a document starts lines with, top to bottom; other fences hide what they hold. */
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
    const start = bytes + (open?.[1].length ?? 0);
    next();
    if (!open) continue;
    const mark = open[2];
    const closing = new RegExp(`^ {0,3}${mark[0]}{${mark.length},}\\s*$`);
    const inner: string[] = [];
    while (at < lines.length && !closing.test(lines[at])) {
      inner.push(`${lines[at]}\n`);
      next();
    }
    if (at < lines.length) next();
    if (DIAGRAM.test(open[3])) found.push({ at: start, source: inner.join("") });
  }
  return found;
};

/** Which block saying `source` was just written: the one the text before did not have at its place. */
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
