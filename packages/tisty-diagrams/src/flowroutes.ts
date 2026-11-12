import { getSmoothStepPath, Position } from "@xyflow/system";
import type { Rect } from "./graphlayout";
import { pillSize } from "./picture";

export interface Route {
  readonly path: string;
  readonly labelX: number;
  readonly labelY: number;
  // Where it leaves and where it arrives, and by which sides, for words written at its ends.
  readonly from: { readonly x: number; readonly y: number };
  readonly to: { readonly x: number; readonly y: number };
  readonly fromSide: Position;
  readonly toSide: Position;
  readonly way: readonly { readonly x: number; readonly y: number }[];
}

export interface Joint {
  readonly from: Rect;
  readonly to: Rect;
  readonly label?: string;
}

/** Where words at an end of a line go: beside it, just off the shape, never over the line or the shape. */
export const endWordAt = (
  at: { readonly x: number; readonly y: number },
  side: Position,
): { x: number; y: number; anchor: "start" | "end" } => {
  if (side === Position.Left) return { x: at.x - 6, y: at.y - 10, anchor: "end" };
  if (side === Position.Right) return { x: at.x + 6, y: at.y - 10, anchor: "start" };
  if (side === Position.Top) return { x: at.x + 6, y: at.y - 10, anchor: "start" };
  return { x: at.x + 6, y: at.y + 10, anchor: "start" };
};

/** The middle of one side of a shape, where its dot sits and its arrows meet it. */
export const sideMiddle = (rect: Rect, side: Position) => {
  if (side === Position.Top) return { x: rect.x + rect.wide / 2, y: rect.y };
  if (side === Position.Bottom) return { x: rect.x + rect.wide / 2, y: rect.y + rect.tall };
  if (side === Position.Left) return { x: rect.x, y: rect.y + rect.tall / 2 };
  return { x: rect.x + rect.wide, y: rect.y + rect.tall / 2 };
};

/** The sides an arrow leaves and enters by: the ones facing each other across the wider gap. */
export const sidesOf = (from: Rect, to: Rect): readonly [Position, Position] => {
  const dx = to.x + to.wide / 2 - (from.x + from.wide / 2);
  const dy = to.y + to.tall / 2 - (from.y + from.tall / 2);
  if (dx === 0 && dy === 0) return [Position.Right, Position.Top];
  if (Math.abs(dx) >= Math.abs(dy))
    return dx >= 0 ? [Position.Right, Position.Left] : [Position.Left, Position.Right];
  return dy >= 0 ? [Position.Bottom, Position.Top] : [Position.Top, Position.Bottom];
};

type Point = { readonly x: number; readonly y: number };

const TURN = 18;
const ROOM = 6;

const stepped = (start: Point, end: Point, leaving: Position, entering: Position, center?: Point) =>
  getSmoothStepPath({
    sourceX: start.x,
    sourceY: start.y,
    sourcePosition: leaving,
    targetX: end.x,
    targetY: end.y,
    targetPosition: entering,
    borderRadius: 6,
    offset: TURN,
    centerX: center?.x,
    centerY: center?.y,
  });

const same = (one: Rect, other: Rect) =>
  one.x === other.x && one.y === other.y && one.wide === other.wide && one.tall === other.tall;

export const overlaps = (one: Rect, other: Rect, room = 0) =>
  one.x < other.x + other.wide + room &&
  one.x + one.wide > other.x - room &&
  one.y < other.y + other.tall + room &&
  one.y + one.tall > other.y - room;

const runOf = (a: Point, b: Point): Rect => ({
  x: Math.min(a.x, b.x),
  y: Math.min(a.y, b.y),
  wide: Math.abs(b.x - a.x),
  tall: Math.abs(b.y - a.y),
});

const free = (way: readonly Point[], shapes: readonly Rect[]) =>
  way.every(
    (one, at) =>
      at === 0 || !shapes.some((shape) => overlaps(runOf(way[at - 1], one), shape, ROOM)),
  );

// Turning halfway is how it always ran; just past its shape or just before the other is how it goes round one in between.
const waysBetween = (start: Point, end: Point, leaving: Position): Point[][] => {
  const flat = leaving === Position.Left || leaving === Position.Right;
  const from = flat ? start.x : start.y;
  const to = flat ? end.x : end.y;
  const ahead = leaving === Position.Right || leaving === Position.Bottom ? 1 : -1;
  if ((to - from) * ahead < TURN * 2) return [];
  const at = (along: number, aside: number): Point =>
    flat ? { x: along, y: aside } : { x: aside, y: along };
  const aside = (one: Point) => (flat ? one.y : one.x);
  return [(from + to) / 2, from + TURN * ahead, to - TURN * ahead].map((turn) => [
    start,
    at(turn, aside(start)),
    at(turn, aside(end)),
    end,
  ]);
};

/** An arrow in right-angled runs between the nearest sides, round any shape in its way; the window and the picture both draw this one. */
export const routeOf = (from: Rect, to: Rect, around: readonly Rect[] = []): Route => {
  const [leaving, entering] = sidesOf(from, to);
  const start = sideMiddle(from, leaving);
  const end = sideMiddle(to, entering);
  const [path, labelX, labelY] = stepped(start, end, leaving, entering);
  const plain = {
    path,
    labelX,
    labelY,
    from: start,
    to: end,
    fromSide: leaving,
    toSide: entering,
    way: [],
  };
  const others = around.filter((one) => !same(one, from) && !same(one, to));
  const ways = waysBetween(start, end, leaving);
  const way = ways.find((one) => free(one, others));
  if (!way) return { ...plain, way: ways[0] ?? [] };
  const turn = { x: (way[1].x + way[2].x) / 2, y: (way[1].y + way[2].y) / 2 };
  const [drawn, x, y] =
    way === ways[0] ? [path, labelX, labelY] : stepped(start, end, leaving, entering, turn);
  return { ...plain, path: drawn, labelX: x, labelY: y, way };
};

const middlesOf = (way: readonly Point[]): Point[] =>
  way.slice(1).map((one, at) => ({ x: (way[at].x + one.x) / 2, y: (way[at].y + one.y) / 2 }));

/** Every arrow of a drawing, each word where it covers no shape and no other arrow's word. */
export const routesOf = (joints: readonly Joint[], shapes: readonly Rect[]): Route[] => {
  const taken: Rect[] = [];
  return joints.map((joint) => {
    const route = routeOf(joint.from, joint.to, shapes);
    if (!joint.label) return route;
    const size = pillSize(joint.label);
    const box = (at: Point): Rect => ({
      x: at.x - size.wide / 2,
      y: at.y - size.tall / 2,
      ...size,
    });
    const spots = [{ x: route.labelX, y: route.labelY }, ...middlesOf(route.way).reverse()];
    const spot =
      spots.find(
        (at) =>
          !shapes.some((one) => overlaps(box(at), one, 2)) &&
          !taken.some((one) => overlaps(box(at), one, 4)),
      ) ?? spots[0];
    taken.push(box(spot));
    return { ...route, labelX: spot.x, labelY: spot.y };
  });
};
