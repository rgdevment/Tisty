import { headerAt } from "./framing";
import type { Direction } from "./graphlayout";

export type Shape =
  | "process"
  | "rounded"
  | "stadium"
  | "subroutine"
  | "database"
  | "circle"
  | "double"
  | "decision"
  | "hexagon"
  | "lean"
  | "leanLeft"
  | "trapezoid"
  | "trapezoidDown"
  | "flag";

export const SHAPES: readonly Shape[] = [
  "process",
  "rounded",
  "stadium",
  "circle",
  "decision",
  "hexagon",
  "database",
  "subroutine",
  "lean",
  "flag",
];

// Opening, the closings it may take with the shape each gives; longest openings first.
const FORMS: readonly (readonly [string, readonly (readonly [string, Shape])[]])[] = [
  ["(((", [[")))", "double"]]],
  ["((", [["))", "circle"]]],
  ["([", [["])", "stadium"]]],
  ["[[", [["]]", "subroutine"]]],
  ["[(", [[")]", "database"]]],
  ["{{", [["}}", "hexagon"]]],
  [
    "[/",
    [
      ["/]", "lean"],
      ["\\]", "trapezoid"],
    ],
  ],
  [
    "[\\",
    [
      ["\\]", "leanLeft"],
      ["/]", "trapezoidDown"],
    ],
  ],
  ["(", [[")", "rounded"]]],
  ["[", [["]", "process"]]],
  ["{", [["}", "decision"]]],
  [">", [["]", "flag"]]],
];

export interface Span {
  readonly line: number;
  readonly start: number;
  readonly end: number;
}

export interface FlowNode {
  readonly id: string;
  readonly text: string;
  readonly shape: Shape | null;
  // Where its words and shape are written, or else where it is first named.
  readonly def: Span | null;
  readonly first: Span;
  readonly group: string | null;
}

export interface FlowEdge {
  readonly from: string;
  readonly to: string;
  readonly arrow: string;
  readonly label: string;
  readonly link: Span;
  // A line that says this one edge and nothing else can be rewritten or taken out whole.
  readonly alone: boolean;
}

export interface FlowGroup {
  readonly id: string;
  readonly title: string;
  readonly open: number;
  readonly close: number;
}

export interface Flow {
  readonly head: readonly string[];
  readonly body: readonly string[];
  readonly tail: readonly string[];
  readonly direction: Direction;
  readonly nodes: readonly FlowNode[];
  readonly edges: readonly FlowEdge[];
  readonly groups: readonly FlowGroup[];
}

interface Named {
  id: string;
  start: number;
  end: number;
  shape: Shape | null;
  text: string | null;
}

interface Linked {
  arrow: string;
  label: string;
  start: number;
  end: number;
}

const ID = /[\p{L}\p{N}_]+(?:-[\p{L}\p{N}_]+)*/uy;
const CLASS = /:::[\w-]+/y;
const SPACE = /\s*/y;
// `-- words -->` and its dotted and thick kin, tried before the bare arrows they begin like.
const WORDED = /(<?)(--|==|-\.)\s+(.+?)\s+(-{2,}[>ox]|-{3,}|={2,}>|={3,}|\.-+>|\.-+)/y;
const BARE = /(<?)(-{2,}|={2,}|-\.+-)([>ox]?)/y;
const PIPED = /\s*\|([^|]*)\|/y;

const at = (pattern: RegExp, said: string, from: number) => {
  pattern.lastIndex = from;
  return pattern.exec(said);
};

const unquoted = (text: string) =>
  (text.startsWith('"') && text.endsWith('"') ? text.slice(1, -1) : text).replace(/#quot;/g, '"');

const shapedAt = (
  said: string,
  from: number,
): { end: number; shape: Shape; text: string } | null => {
  for (const [open, closes] of FORMS) {
    if (!said.startsWith(open, from)) continue;
    const inner = from + open.length;
    const quoted = said[inner] === '"' ? said.indexOf('"', inner + 1) + 1 : inner;
    if (quoted === 0) return null;
    let best: { end: number; shape: Shape; text: string } | null = null;
    for (const [close, shape] of closes) {
      const found = said.indexOf(close, quoted);
      if (found >= 0 && (!best || found < best.end))
        best = { end: found + close.length, shape, text: unquoted(said.slice(inner, found)) };
    }
    return best;
  }
  return null;
};

const nodeAt = (said: string, from: number): Named | null => {
  const id = at(ID, said, from);
  if (!id) return null;
  let end = from + id[0].length;
  const shaped = shapedAt(said, end);
  if (shaped) end = shaped.end;
  const kind = at(CLASS, said, end);
  if (kind) end += kind[0].length;
  return { id: id[0], start: from, end, shape: shaped?.shape ?? null, text: shaped?.text ?? null };
};

// `-- words -->` says the arrow its closing says; a dotted one closes with `.->` or `.-`.
const wordedArrow = (back: string, close: string) => {
  if (!close.startsWith(".")) return `${back}${close}`;
  return `${back}${close.endsWith(">") ? "-.->" : "-.-"}`;
};

const linkAt = (said: string, from: number): Linked | null => {
  const worded = at(WORDED, said, from);
  if (worded)
    return {
      arrow: wordedArrow(worded[1], worded[4]),
      label: worded[3],
      start: from,
      end: from + worded[0].length,
    };
  const bare = at(BARE, said, from);
  if (!bare) return null;
  const end = from + bare[0].length;
  const piped = at(PIPED, said, end);
  return {
    arrow: bare[0],
    label: piped ? piped[1].trim() : "",
    start: from,
    end: piped ? end + piped[0].length : end,
  };
};

interface Statement {
  groups: Named[][];
  links: Linked[];
}

const skip = (said: string, from: number) => from + (at(SPACE, said, from)?.[0].length ?? 0);

// A group is nodes joined by `&`; a statement is groups joined by links; `;` starts another.
const groupAt = (said: string, from: number): { nodes: Named[]; end: number } | null => {
  const nodes: Named[] = [];
  let here = from;
  for (;;) {
    const node = nodeAt(said, skip(said, here));
    if (!node) return null;
    nodes.push(node);
    here = skip(said, node.end);
    if (said[here] !== "&") return { nodes, end: node.end };
    here += 1;
  }
};

/** The statements of a line, or `null` when it holds something the window does not read. */
export const statementsOf = (said: string): Statement[] | null => {
  const all: Statement[] = [];
  let here = skip(said, 0);
  while (here < said.length) {
    const first = groupAt(said, here);
    if (!first) return null;
    const statement: Statement = { groups: [first.nodes], links: [] };
    here = skip(said, first.end);
    for (;;) {
      const link = linkAt(said, here);
      if (!link) break;
      const next = groupAt(said, skip(said, link.end));
      if (!next) return null;
      statement.links.push(link);
      statement.groups.push(next.nodes);
      here = skip(said, next.end);
    }
    all.push(statement);
    if (said[here] === ";") here = skip(said, here + 1);
    else if (here < said.length) return null;
  }
  return all;
};

const KEPT =
  /^\s*(%%|classDef\b|class\b|style\b|linkStyle\b|click\b|accTitle\b|accDescr\b|direction\b)/;
const SUBGRAPH = /^(\s*)subgraph\s+(.+?)\s*$/;
const END = /^\s*end\s*;?\s*$/;
const DIRECTION = /^\s*(flowchart|graph)(?:-[\w-]+)?(?:\s+(TB|TD|BT|LR|RL))?\b/;

const groupNamed = (said: string): { id: string; title: string } => {
  const found = /^([\p{L}\p{N}_-]+)\s*\[\s*("?)(.*?)\2\s*\]$/u.exec(said);
  if (found) return { id: found[1], title: found[3] };
  return { id: said.replace(/^"|"$/g, ""), title: said.replace(/^"|"$/g, "") };
};

type Reading = {
  nodes: Map<string, FlowNode>;
  edges: FlowEdge[];
  groups: FlowGroup[];
  open: { id: string; title: string; line: number }[];
};

const noted = (reading: Reading, node: Named, line: number) => {
  const span = { line, start: node.start, end: node.end };
  const group = reading.open[reading.open.length - 1]?.id ?? null;
  const known = reading.nodes.get(node.id);
  // As Mermaid does, the last words and shape given win.
  const def = node.shape ? span : (known?.def ?? null);
  reading.nodes.set(node.id, {
    id: node.id,
    text: node.text ?? known?.text ?? node.id,
    shape: node.shape ?? known?.shape ?? null,
    def,
    first: known?.first ?? span,
    // As Mermaid does, a node listed inside a subgraph belongs to it, wherever it was named before.
    group: group ?? known?.group ?? null,
  });
};

const readLine = (reading: Reading, said: string, line: number): boolean => {
  if (!said.trim() || KEPT.test(said)) return true;
  const sub = SUBGRAPH.exec(said);
  if (sub) {
    reading.open.push({ ...groupNamed(sub[2]), line });
    return true;
  }
  if (END.test(said)) {
    const open = reading.open.pop();
    if (!open) return false;
    reading.groups.push({ id: open.id, title: open.title, open: open.line, close: line });
    return true;
  }
  const statements = statementsOf(said);
  if (!statements) return false;
  const alone =
    statements.length === 1 &&
    statements[0].links.length === 1 &&
    statements[0].groups.every((one) => one.length === 1);
  for (const statement of statements) {
    for (const group of statement.groups) for (const node of group) noted(reading, node, line);
    statement.links.forEach((link, index) => {
      for (const from of statement.groups[index])
        for (const to of statement.groups[index + 1])
          reading.edges.push({
            from: from.id,
            to: to.id,
            arrow: link.arrow,
            label: link.label,
            link: { line, start: link.start, end: link.end },
            alone,
          });
    });
  }
  return true;
};

/** The flowchart a diagram draws, or `null` when it says something the window could not keep as written. */
export const readFlow = (source: string): Flow | null => {
  const lines = source.replace(/\r/g, "").split("\n");
  const opening = headerAt(lines, DIRECTION);
  if (opening < 0) return null;
  const said = DIRECTION.exec(lines[opening])?.[2] ?? "TB";
  const direction = (said === "TD" ? "TB" : said) as Direction;
  let end = lines.length;
  while (end > opening + 1 && !lines[end - 1].trim()) end -= 1;
  const body = lines.slice(opening + 1, end);
  const reading: Reading = { nodes: new Map(), edges: [], groups: [], open: [] };
  for (const [line, text] of body.entries()) if (!readLine(reading, text, line)) return null;
  if (reading.open.length) return null;
  return {
    head: lines.slice(0, opening + 1),
    body,
    tail: lines.slice(end),
    direction,
    nodes: [...reading.nodes.values()],
    edges: reading.edges,
    groups: reading.groups,
  };
};

export const writeFlow = (flow: Pick<Flow, "head" | "body" | "tail">): string =>
  [...flow.head, ...flow.body, ...flow.tail].join("\n");

const OPENING: Record<Shape, readonly [string, string]> = {
  process: ["[", "]"],
  rounded: ["(", ")"],
  stadium: ["([", "])"],
  subroutine: ["[[", "]]"],
  database: ["[(", ")]"],
  circle: ["((", "))"],
  double: ["(((", ")))"],
  decision: ["{", "}"],
  hexagon: ["{{", "}}"],
  lean: ["[/", "/]"],
  leanLeft: ["[\\", "\\]"],
  trapezoid: ["[/", "\\]"],
  trapezoidDown: ["[\\", "/]"],
  flag: [">", "]"],
};

/** A node as Mermaid reads it back: its id, then its words in its shape, quoted when they would break it. */
export const formedNode = (id: string, text: string, shape: Shape): string => {
  const [open, close] = OPENING[shape];
  const plain = text.trim() === text && !/["()[\]{}|<>;]/.test(text);
  return `${id}${open}${plain ? text : `"${text.replace(/"/g, "#quot;")}"`}${close}`;
};
