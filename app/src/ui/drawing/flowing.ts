import { type Flow, type FlowEdge, formedNode, type Shape, type Span, statementsOf } from "./flow";
import type { Direction } from "./graphlayout";
import { names } from "./naming";

/** What is chosen: a node by its id, an edge by its place among them, a group by its id. */
export type FlowPicked =
  | { readonly kind: "node"; readonly id: string }
  | { readonly kind: "edge"; readonly index: number }
  | { readonly kind: "group"; readonly id: string }
  | null;

/** A change is new text for the body of the flowchart, read again by whoever shows it. */
export interface FlowAltered {
  readonly body: readonly string[];
  readonly picked: FlowPicked;
}

const spliced = (body: readonly string[], span: Span, said: string) =>
  body.map((line, at) =>
    at === span.line ? `${line.slice(0, span.start)}${said}${line.slice(span.end)}` : line,
  );

const indentOf = (line: string | undefined) =>
  line?.slice(0, line.length - line.trimStart().length) ?? "";

// New lines go at the end, or inside a group just before its `end`, indented like what is around them.
const inserted = (
  flow: Flow,
  lines: readonly string[],
  group: string | null,
): readonly string[] => {
  const inside = flow.groups.find((one) => one.id === group);
  if (inside) {
    const indent = `${indentOf(flow.body[inside.open])}  `;
    return [
      ...flow.body.slice(0, inside.close),
      ...lines.map((one) => `${indent}${one}`),
      ...flow.body.slice(inside.close),
    ];
  }
  const indent = indentOf(flow.body.find((one) => one.trim())) || "  ";
  return [...flow.body, ...lines.map((one) => `${indent}${one}`)];
};

const unique = (wanted: string, taken: readonly string[]) => {
  let n = 1;
  while (taken.includes(`${wanted}${n}`)) n += 1;
  return `${wanted}${n}`;
};

export const nodeChanged = (
  flow: Flow,
  id: string,
  change: { text?: string; shape?: Shape },
): FlowAltered => {
  const node = flow.nodes.find((one) => one.id === id);
  if (!node) return { body: flow.body, picked: null };
  const text = change.text?.trim() ? change.text.trim() : node.text;
  const shape = change.shape ?? node.shape ?? "process";
  const span = node.def ?? node.first;
  // A `:::class` after the shape stays as it was written.
  const kind = /:::[\w-]+$/.exec(flow.body[span.line].slice(span.start, span.end))?.[0] ?? "";
  return {
    body: spliced(flow.body, span, `${formedNode(id, text, shape)}${kind}`),
    picked: { kind: "node", id },
  };
};

export const nodeAdded = (flow: Flow, text: string, group: string | null): FlowAltered => {
  const id = unique(
    "n",
    flow.nodes.map((one) => one.id),
  );
  return {
    body: inserted(flow, [formedNode(id, text, "process")], group),
    picked: { kind: "node", id },
  };
};

/** An arrow from one node to another, or the one already there. */
export const linked = (flow: Flow, from: string, to: string): FlowAltered => {
  const already = flow.edges.findIndex((one) => one.from === from && one.to === to);
  if (already >= 0) return { body: flow.body, picked: { kind: "edge", index: already } };
  return {
    body: inserted(flow, [`${from} --> ${to}`], null),
    picked: { kind: "edge", index: flow.edges.length },
  };
};

export const ARROWS = ["-->", "---", "-.->", "==>"] as const;

const linkText = (arrow: string, label: string) =>
  label ? `${arrow}|${label.replace(/\|/g, "/")}|` : arrow;

export const edgeChanged = (
  flow: Flow,
  index: number,
  change: { label?: string; arrow?: string },
): FlowAltered => {
  const edge = flow.edges[index];
  if (!edge) return { body: flow.body, picked: null };
  const said = linkText(change.arrow ?? edge.arrow, (change.label ?? edge.label).trim());
  return { body: spliced(flow.body, edge.link, said), picked: { kind: "edge", index } };
};

// Only a line that says just this edge can go; an edge in a chain would take the chain with it.
export const edgeRemovable = (edge: FlowEdge) => edge.alone;

// What a line held besides its edge stays when the line goes: a node written there, or one named nowhere else.
const leftBehind = (
  flow: Flow,
  at: number,
  ids: readonly string[],
  gone: ReadonlySet<number>,
  placed: Set<string>,
) => {
  const indent = indentOf(flow.body[at]);
  return ids.flatMap((id) => {
    const node = flow.nodes.find((one) => one.id === id);
    if (!node || placed.has(id)) return [];
    const elsewhere = flow.body.some((line, other) => !gone.has(other) && names(line, id));
    if (node.def?.line !== at && elsewhere) return [];
    placed.add(id);
    if (node.def?.line !== at) return [`${indent}${id}`];
    const kind = /:::[\w-]+$/.exec(flow.body[at].slice(node.def.start, node.def.end))?.[0] ?? "";
    return [`${indent}${formedNode(id, node.text, node.shape ?? "process")}${kind}`];
  });
};

const STYLED = /^(\s*linkStyle\s+)([\d\s,]+?)(\s+\S.*)$/;

// `linkStyle` names edges by their order, so the ones after a removed edge move down by one each.
const restyled = (line: string, removed: readonly number[]): string | null => {
  const found = STYLED.exec(line);
  if (!found) return line;
  const kept = found[2]
    .split(",")
    .map((one) => Number(one.trim()))
    .filter((one) => !removed.includes(one))
    .map((one) => one - removed.filter((gone) => gone < one).length);
  return kept.length ? `${found[1]}${kept.join(",")}${found[3]}` : null;
};

// Lines replaced or dropped by place, `linkStyle` renumbered for the edges that went.
const rewritten = (
  flow: Flow,
  instead: ReadonlyMap<number, readonly string[]>,
  removed: readonly number[],
) =>
  flow.body.flatMap((line, at) => {
    const replaced = instead.get(at);
    if (replaced) return replaced;
    const styled = removed.length ? restyled(line, removed) : line;
    return styled === null ? [] : [styled];
  });

export const edgeRemoved = (flow: Flow, index: number): FlowAltered | null => {
  const edge = flow.edges[index];
  if (!edge || !edgeRemovable(edge)) return null;
  const at = edge.link.line;
  const kept = leftBehind(flow, at, [edge.from, edge.to], new Set([at]), new Set());
  return { body: rewritten(flow, new Map([[at, kept]]), [index]), picked: null };
};

// A line goes with a node when it names that node alone: its own line, a lone edge from or to it, its style.
const goesWith = (flow: Flow, at: number, id: string): boolean | null => {
  const line = flow.body[at];
  if (!names(line, id)) return false;
  if (/^\s*(style|click)\s/.test(line))
    return new RegExp(`^\\s*(style|click)\\s+${id}\\b`).test(line);
  if (/^\s*(%%|classDef|linkStyle|accTitle|accDescr|direction|subgraph|end)\b/.test(line))
    return false;
  if (/^\s*class\s/.test(line)) return null;
  const statements = statementsOf(line);
  if (statements?.length !== 1) return null;
  const [statement] = statements;
  const named = statement.groups.flat().map((one) => one.id);
  if (statement.links.length === 0) return named.every((one) => one === id) ? true : null;
  return statement.links.length === 1 && named.length === 2 ? true : null;
};

/** A node with its lone edges and its own lines; refused while a chain or a shared line still names it. */
export const nodeRemoved = (flow: Flow, id: string): FlowAltered | null => {
  const going = flow.body.map((_, at) => goesWith(flow, at, id));
  if (going.includes(null)) return null;
  const gone = new Set(going.flatMap((goes, at) => (goes ? [at] : [])));
  const placed = new Set([id]);
  const instead = new Map(
    [...gone].map((at) => {
      const others = flow.edges
        .filter((one) => one.link.line === at)
        .flatMap((one) => [one.from, one.to]);
      return [at, leftBehind(flow, at, others, gone, placed)] as const;
    }),
  );
  const removed = flow.edges.flatMap((one, index) => (gone.has(one.link.line) ? [index] : []));
  return { body: rewritten(flow, instead, removed), picked: null };
};

export const groupAdded = (flow: Flow, title: string): FlowAltered => {
  const id = unique(
    "g",
    flow.groups.map((one) => one.id),
  );
  const lines = [`subgraph ${id} [${title}]`, "end"];
  return { body: inserted(flow, lines, null), picked: { kind: "group", id } };
};

export const groupRenamed = (flow: Flow, id: string, title: string): FlowAltered => {
  const group = flow.groups.find((one) => one.id === id);
  if (!group || !title.trim()) return { body: flow.body, picked: { kind: "group", id } };
  const plain = !/["[\]]/.test(title);
  const said = `${indentOf(flow.body[group.open])}subgraph ${id} [${plain ? title.trim() : `"${title.replace(/"/g, "#quot;").trim()}"`}]`;
  return {
    body: flow.body.map((line, at) => (at === group.open ? said : line)),
    picked: { kind: "group", id },
  };
};

/** A group taken apart: its frame goes, what was inside stays. */
export const groupRemoved = (flow: Flow, id: string): FlowAltered => {
  const group = flow.groups.find((one) => one.id === id);
  if (!group) return { body: flow.body, picked: null };
  return {
    body: flow.body.filter((_, at) => at !== group.open && at !== group.close),
    picked: null,
  };
};

/** A node moved into a group, or out of every group, by moving the lines that name it alone. */
export const nodeMoved = (flow: Flow, id: string, group: string | null): FlowAltered | null => {
  const node = flow.nodes.find((one) => one.id === id);
  if (!node || node.group === group)
    return node ? { body: flow.body, picked: { kind: "node", id } } : null;
  const own = flow.body
    .map((line, at) => {
      const statements = statementsOf(line);
      return statements?.length === 1 &&
        statements[0].links.length === 0 &&
        statements[0].groups.flat().every((one) => one.id === id)
        ? at
        : -1;
    })
    .filter((at) => at >= 0);
  // A node named inside its group by an edge cannot leave it without that edge.
  const named = flow.body.some(
    (line, at) =>
      !own.includes(at) &&
      names(line, id) &&
      insideOf(flow, at) === node.group &&
      node.group !== null,
  );
  if (named) return null;
  const kept = { ...flow, body: flow.body.filter((_, at) => !own.includes(at)) };
  const said =
    node.def && own.includes(node.def.line)
      ? formedNode(id, node.text, node.shape ?? "process")
      : id;
  const reread = { ...kept, groups: kept.groups.map((one) => shifted(one, own)) };
  return { body: inserted(reread, [said], group), picked: { kind: "node", id } };
};

const insideOf = (flow: Flow, at: number) =>
  [...flow.groups].reverse().find((one) => one.open < at && at < one.close)?.id ?? null;

const shifted = <T extends { open: number; close: number }>(
  group: T,
  gone: readonly number[],
): T => ({
  ...group,
  open: group.open - gone.filter((one) => one < group.open).length,
  close: group.close - gone.filter((one) => one < group.close).length,
});

export const directionChanged = (flow: Flow, direction: Direction): string[] => {
  const head = [...flow.head];
  const at = head.length - 1;
  // `flowchart-elk` keeps its layout word; only the direction after it changes.
  head[at] = head[at].replace(
    /^(\s*)(flowchart|graph)(-[\w-]+)?(\s+(TB|TD|BT|LR|RL))?/,
    `$1$2$3 ${direction}`,
  );
  return head;
};
