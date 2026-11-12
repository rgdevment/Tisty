import type { Size } from "./framing";

export type Direction = "TB" | "BT" | "LR" | "RL";

export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly wide: number;
  readonly tall: number;
}

export interface Laid {
  readonly at: ReadonlyMap<string, Rect>;
  readonly size: Size;
}

const GAP = 56;
export const MARGIN = 40;

// Each node's layer is the longest way to it from a node nothing points at; a loop is cut where it closes.
const ranksOf = (ids: readonly string[], edges: readonly (readonly [string, string])[]) => {
  const out = new Map<string, string[]>(ids.map((id) => [id, []]));
  for (const [from, to] of edges) if (from !== to && out.has(to)) out.get(from)?.push(to);
  const forward = new Map<string, string[]>(ids.map((id) => [id, []]));
  const walking = new Set<string>();
  const seen = new Set<string>();
  const walk = (id: string) => {
    seen.add(id);
    walking.add(id);
    for (const next of out.get(id) ?? []) {
      if (walking.has(next)) continue;
      forward.get(id)?.push(next);
      if (!seen.has(next)) walk(next);
    }
    walking.delete(id);
  };
  for (const id of ids) if (!seen.has(id)) walk(id);
  const rank = new Map<string, number>(ids.map((id) => [id, 0]));
  const waiting = new Map<string, number>(ids.map((id) => [id, 0]));
  for (const nexts of forward.values())
    for (const next of nexts) waiting.set(next, (waiting.get(next) ?? 0) + 1);
  const ready = ids.filter((id) => waiting.get(id) === 0);
  for (let at = 0; at < ready.length; at++) {
    const id = ready[at];
    for (const next of forward.get(id) ?? []) {
      rank.set(next, Math.max(rank.get(next) ?? 0, (rank.get(id) ?? 0) + 1));
      const left = (waiting.get(next) ?? 0) - 1;
      waiting.set(next, left);
      if (left === 0) ready.push(next);
    }
  }
  return rank;
};

export interface Room {
  readonly aside: ReadonlyMap<string, number>;
  readonly along: readonly number[];
}

/** Layers along the direction the diagram says, nodes side by side in the order they were written. */
export const layered = (
  ids: readonly string[],
  edges: readonly (readonly [string, string])[],
  sizes: ReadonlyMap<string, Size>,
  direction: Direction,
  room: Room = { aside: new Map(), along: [] },
): Laid => {
  const rank = ranksOf(ids, edges);
  const layers: string[][] = [];
  for (const id of ids) {
    const layer = rank.get(id) ?? 0;
    layers[layer] = [...(layers[layer] ?? []), id];
  }
  const across = direction === "LR" || direction === "RL";
  const along = (size: Size) => (across ? size.wide : size.tall);
  const aside = (size: Size) => (across ? size.tall : size.wide);
  const sizeOf = (id: string) => sizes.get(id) ?? { wide: 120, tall: 40 };
  const deep = layers.map((layer) => Math.max(0, ...layer.map((id) => along(sizeOf(id)))));
  const apart = (one: string, next: string) =>
    Math.max(GAP, ((room.aside.get(one) ?? 0) + (room.aside.get(next) ?? 0)) / 2);
  const crossing = (low: number, [from, to]: readonly [string, string]) => {
    const [a, b] = [rank.get(from) ?? 0, rank.get(to) ?? 0];
    return Math.min(a, b) <= low && low < Math.max(a, b);
  };
  const between = (low: number) =>
    Math.max(
      GAP * 1.4,
      ...edges.map((edge, at) => (crossing(low, edge) && room.along[at] ? room.along[at] + 48 : 0)),
    );
  const broad = layers.map((layer) =>
    layer.reduce(
      (sum, id, at) => sum + aside(sizeOf(id)) + (at > 0 ? apart(layer[at - 1], id) : 0),
      0,
    ),
  );
  const widest = Math.max(0, ...broad);
  const order =
    direction === "BT" || direction === "RL" ? [...layers.keys()].reverse() : [...layers.keys()];
  const at = new Map<string, Rect>();
  let depth = MARGIN;
  order.forEach((index, step) => {
    const layer = layers[index] ?? [];
    let side = MARGIN + (widest - broad[index]) / 2;
    layer.forEach((id, place) => {
      const size = sizeOf(id);
      const middle = depth + (deep[index] - along(size)) / 2;
      at.set(id, across ? { x: middle, y: side, ...size } : { x: side, y: middle, ...size });
      side += aside(size) + (place + 1 < layer.length ? apart(id, layer[place + 1]) : 0);
    });
    const next = order[step + 1];
    depth += deep[index] + (next === undefined ? GAP * 1.4 : between(Math.min(index, next)));
  });
  const rects = [...at.values()];
  const wide = Math.max(MARGIN * 2, ...rects.map((one) => one.x + one.wide + MARGIN));
  const tall = Math.max(MARGIN * 2, ...rects.map((one) => one.y + one.tall + MARGIN));
  return { at, size: { wide, tall } };
};

/** The box around some others, with room on top for a title. */
export const around = (rects: readonly Rect[], pad = 18, title = 22): Rect | null => {
  if (!rects.length) return null;
  const left = Math.min(...rects.map((one) => one.x)) - pad;
  const top = Math.min(...rects.map((one) => one.y)) - pad - title;
  const right = Math.max(...rects.map((one) => one.x + one.wide)) + pad;
  const bottom = Math.max(...rects.map((one) => one.y + one.tall)) + pad;
  return { x: left, y: top, wide: right - left, tall: bottom - top };
};
