import type { Point } from "./zones";

export type Way = "up" | "down" | "left" | "right";

const HEADING: Record<Way, Point> = {
  up: [0, -1],
  down: [0, 1],
  left: [-1, 0],
  right: [1, 0],
};

const CONE = Math.cos(Math.PI / 3);

export const wayOf = (key: string): Way | undefined =>
  (
    ({ ArrowUp: "up", ArrowDown: "down", ArrowLeft: "left", ArrowRight: "right" }) as Record<
      string,
      Way
    >
  )[key];

const nearestIn = (
  from: Point,
  way: Way,
  candidates: Iterable<number>,
  at: (node: number) => Point | undefined,
  skip: number,
): number | undefined => {
  const [hx, hy] = HEADING[way];
  let best: number | undefined;
  let bestDistance = Infinity;
  for (const node of candidates) {
    if (node === skip) continue;
    const there = at(node);
    if (!there) continue;
    const dx = there[0] - from[0];
    const dy = there[1] - from[1];
    const distance = Math.hypot(dx, dy);
    if (distance === 0 || (dx * hx + dy * hy) / distance < CONE) continue;
    if (distance < bestDistance) {
      best = node;
      bestDistance = distance;
    }
  }
  return best;
};

// A neighbour that way comes first, so the keys walk the lines before they jump across the map.
export function stepped(
  from: number,
  way: Way,
  at: (node: number) => Point | undefined,
  beside: readonly number[],
  count: number,
): number | undefined {
  const here = at(from);
  if (!here) return undefined;
  return (
    nearestIn(here, way, beside, at, from) ??
    nearestIn(
      here,
      way,
      Array.from({ length: count }, (_, n) => n),
      at,
      from,
    )
  );
}
