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
const MARGIN = 40;

// Each node's layer is the longest way to it from a node nothing points at; a loop is cut where it closes.
const ranksOf = (ids: readonly string[], edges: readonly (readonly [string, string])[]) => {
  const out = new Map<string, string[]>(ids.map((id) => [id, []]));
  for (const [from, to] of edges) if (from !== to) out.get(from)?.push(to);
  const rank = new Map<string, number>(ids.map((id) => [id, 0]));
  const walking = new Set<string>();
  const done = new Set<string>();
  const walk = (id: string) => {
    if (done.has(id) || walking.has(id)) return;
    walking.add(id);
    for (const next of out.get(id) ?? []) {
      if (walking.has(next)) continue;
      rank.set(next, Math.max(rank.get(next) ?? 0, (rank.get(id) ?? 0) + 1));
      done.delete(next);
      walk(next);
    }
    walking.delete(id);
    done.add(id);
  };
  for (const id of ids) walk(id);
  return rank;
};

/** Layers along the direction the diagram says, nodes side by side in the order they were written. */
export const layered = (
  ids: readonly string[],
  edges: readonly (readonly [string, string])[],
  sizes: ReadonlyMap<string, Size>,
  direction: Direction,
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
  const broad = layers.map((layer) =>
    layer.reduce((sum, id) => sum + aside(sizeOf(id)), Math.max(0, layer.length - 1) * GAP),
  );
  const widest = Math.max(0, ...broad);
  const order =
    direction === "BT" || direction === "RL" ? [...layers.keys()].reverse() : [...layers.keys()];
  const at = new Map<string, Rect>();
  let depth = MARGIN;
  for (const index of order) {
    const layer = layers[index] ?? [];
    let side = MARGIN + (widest - broad[index]) / 2;
    for (const id of layer) {
      const size = sizeOf(id);
      const middle = depth + (deep[index] - along(size)) / 2;
      at.set(id, across ? { x: middle, y: side, ...size } : { x: side, y: middle, ...size });
      side += aside(size) + GAP;
    }
    depth += deep[index] + GAP * 1.4;
  }
  const rects = [...at.values()];
  const wide = Math.max(MARGIN * 2, ...rects.map((one) => one.x + one.wide + MARGIN));
  const tall = Math.max(MARGIN * 2, ...rects.map((one) => one.y + one.tall + MARGIN));
  return { at, size: { wide, tall } };
};

/** Where a line from one box's middle to another's leaves the first box's edge. */
export const edgeOf = (from: Rect, to: Rect) => {
  const cx = from.x + from.wide / 2;
  const cy = from.y + from.tall / 2;
  const dx = to.x + to.wide / 2 - cx;
  const dy = to.y + to.tall / 2 - cy;
  if (!dx && !dy) return { x: cx, y: cy };
  const scale = Math.min(
    dx ? from.wide / 2 / Math.abs(dx) : Number.POSITIVE_INFINITY,
    dy ? from.tall / 2 / Math.abs(dy) : Number.POSITIVE_INFINITY,
  );
  return { x: cx + dx * scale, y: cy + dy * scale };
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
