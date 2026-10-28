import type { Flow } from "./flow";
import {
  edgeRemoved,
  type FlowAltered,
  type FlowPicked,
  groupRemoved,
  inserted,
  nodeRemoved,
  placed,
  reread,
  unique,
} from "./flowing";
import { formedNode } from "./flowshapes";
import { looksOf, nodeStyled } from "./flowstyles";
import { around, type Rect } from "./graphlayout";
import { flowLaid, lanesOf } from "./scenes";

export type Alignment = "left" | "center" | "right" | "top" | "middle" | "bottom";

export const ALIGNMENTS: readonly Alignment[] = [
  "left",
  "center",
  "right",
  "top",
  "middle",
  "bottom",
];

const settled = (flow: Flow, picked: FlowPicked): FlowAltered => ({
  body: flow.body,
  places: flow.places,
  picked,
});

/** Several nodes taken out at once; refused whole when one of them cannot go. */
export const nodesRemoved = (flow: Flow, ids: readonly string[]): FlowAltered | null => {
  let now = flow;
  for (const id of ids) {
    const made = nodeRemoved(now, id);
    if (!made) return null;
    now = reread(now, made);
  }
  return settled(now, null);
};

/** What taking out the chosen thing leaves, or `null` while it cannot go. */
export const pickedRemoved = (flow: Flow, picked: FlowPicked): FlowAltered | null => {
  if (picked?.kind === "node") return nodeRemoved(flow, picked.id);
  if (picked?.kind === "edge") return edgeRemoved(flow, picked.index);
  if (picked?.kind === "group") return groupRemoved(flow, picked.id);
  return picked ? nodesRemoved(flow, picked.ids) : null;
};

const OFFSET = 24;

/** A copy of a node just beside it, with its shape, its keys and its colours, in its group. */
export const nodeDuplicated = (
  flow: Flow,
  id: string,
  at: Rect | undefined,
): FlowAltered | null => {
  const node = flow.nodes.find((one) => one.id === id);
  if (!node) return null;
  const copy = unique(
    "n",
    flow.nodes.map((one) => one.id),
  );
  const body = inserted(flow, [formedNode(copy, node.text, node.shape, node.keys)], node.group);
  const places = at ? placed(flow, copy, { x: at.x + OFFSET, y: at.y + OFFSET }) : flow.places;
  let now = reread(flow, { body, places, picked: null });
  const look = looksOf(flow).get(id);
  if (look) now = reread(now, nodeStyled(now, copy, look));
  return settled(now, { kind: "node", id: copy });
};

type Edges = { left: number; right: number; top: number; bottom: number };

const linedUp = (how: Alignment, rect: Rect, edges: Edges) => {
  switch (how) {
    case "left":
      return { x: edges.left, y: rect.y };
    case "center":
      return { x: (edges.left + edges.right) / 2 - rect.wide / 2, y: rect.y };
    case "right":
      return { x: edges.right - rect.wide, y: rect.y };
    case "top":
      return { x: rect.x, y: edges.top };
    case "middle":
      return { x: rect.x, y: (edges.top + edges.bottom) / 2 - rect.tall / 2 };
    default:
      return { x: rect.x, y: edges.bottom - rect.tall };
  }
};

/** Where each shape goes to line up with the others on one of its edges or its middle. */
export const alignedMoves = (
  rects: ReadonlyMap<string, Rect>,
  ids: readonly string[],
  how: Alignment,
): { id: string; x: number; y: number }[] => {
  const chosen = ids.flatMap((id) => {
    const rect = rects.get(id);
    return rect ? [{ id, rect }] : [];
  });
  const all = chosen.map((one) => one.rect);
  const edges = {
    left: Math.min(...all.map((one) => one.x)),
    right: Math.max(...all.map((one) => one.x + one.wide)),
    top: Math.min(...all.map((one) => one.y)),
    bottom: Math.max(...all.map((one) => one.y + one.tall)),
  };
  return chosen.map(({ id, rect }) => ({ id, ...linedUp(how, rect, edges) }));
};

const LANE_PAD = 24;

/** Every shape laid out again by the window, and each lane drawn around the shapes in it. */
export const relaid = (flow: Flow): FlowAltered => {
  const lanes = new Set(lanesOf(flow).map((one) => one.id));
  const auto = flowLaid({ ...flow, places: [] });
  let now: Flow = { ...flow, places: flow.places.filter((one) => lanes.has(one.id)) };
  for (const [id, rect] of auto)
    now = { ...now, places: placed(now, id, { x: rect.x, y: rect.y }) };
  for (const lane of lanes) {
    const inside = flow.nodes
      .filter((one) => one.group === lane)
      .flatMap((one) => auto.get(one.id) ?? []);
    const box = around(inside, LANE_PAD, LANE_PAD);
    if (box) now = { ...now, places: placed(now, lane, box) };
  }
  return settled(now, null);
};
