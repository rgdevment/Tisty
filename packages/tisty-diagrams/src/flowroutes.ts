import { getSmoothStepPath, Position } from "@xyflow/system";
import { MARGIN, type Rect } from "./graphlayout";
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
  readonly startLabel?: string;
  readonly endLabel?: string;
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

/** The sides an arrow leaves and enters by: the ones facing each other across the wider gap, when that gap leaves room for a turn. */
export const sidesOf = (from: Rect, to: Rect): readonly [Position, Position] => {
  const dx = to.x + to.wide / 2 - (from.x + from.wide / 2);
  const dy = to.y + to.tall / 2 - (from.y + from.tall / 2);
  if (dx === 0 && dy === 0) return [Position.Right, Position.Top];
  const across = dx >= 0 ? to.x - (from.x + from.wide) : from.x - (to.x + to.wide);
  const down = dy >= 0 ? to.y - (from.y + from.tall) : from.y - (to.y + to.tall);
  const roomy = (gap: number) => gap >= TURN * 2;
  const flat =
    Math.abs(dx) >= Math.abs(dy) ? roomy(across) || !roomy(down) : roomy(across) && !roomy(down);
  if (flat) return dx >= 0 ? [Position.Right, Position.Left] : [Position.Left, Position.Right];
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

const axisOf = (start: Point, end: Point, leaving: Position) => {
  const flat = leaving === Position.Left || leaving === Position.Right;
  const from = flat ? start.x : start.y;
  const to = flat ? end.x : end.y;
  const ahead = leaving === Position.Right || leaving === Position.Bottom ? 1 : -1;
  if ((to - from) * ahead < TURN * 2) return null;
  return {
    flat,
    near: from + TURN * ahead,
    far: to - TURN * ahead,
    middle: (from + to) / 2,
    at: (along: number, aside: number): Point =>
      flat ? { x: along, y: aside } : { x: aside, y: along },
    aside: (one: Point) => (flat ? one.y : one.x),
    alongOf: (one: Rect) => (flat ? [one.x, one.x + one.wide] : [one.y, one.y + one.tall]),
    asideOf: (one: Rect) => (flat ? [one.y, one.y + one.tall] : [one.x, one.x + one.wide]),
  };
};

// Turning halfway is how it always ran; just past its shape or just before the other is how it goes round one in between.
const waysBetween = (start: Point, end: Point, leaving: Position): Point[][] => {
  const axis = axisOf(start, end, leaving);
  if (!axis) return [];
  return [axis.middle, axis.near, axis.far].map((turn) => [
    start,
    axis.at(turn, axis.aside(start)),
    axis.at(turn, axis.aside(end)),
    end,
  ]);
};

const detoursBetween = (
  start: Point,
  end: Point,
  leaving: Position,
  shapes: readonly Rect[],
  within: Rect | null,
): Point[][] => {
  const axis = axisOf(start, end, leaving);
  if (!axis) return [];
  const { near, far, at, aside } = axis;
  const [low, high] = near < far ? [near, far] : [far, near];
  const middle = (aside(start) + aside(end)) / 2;
  const [first, last] = within ? axis.asideOf(within) : [-Infinity, Infinity];
  return shapes
    .filter((one) => {
      const [from, to] = axis.alongOf(one);
      return from < high && to > low;
    })
    .flatMap((one) => {
      const [from, to] = axis.asideOf(one);
      return [from - TURN, to + TURN];
    })
    .filter((lane) => lane - ROOM >= first && lane + ROOM <= last)
    .sort((one, other) => Math.abs(one - middle) - Math.abs(other - middle))
    .map((lane) => [
      start,
      at(near, aside(start)),
      at(near, lane),
      at(far, lane),
      at(far, aside(end)),
      end,
    ]);
};

const toward = (from: Point, to: Point, by: number): Point => {
  const length = Math.hypot(to.x - from.x, to.y - from.y) || 1;
  return {
    x: from.x + ((to.x - from.x) / length) * by,
    y: from.y + ((to.y - from.y) / length) * by,
  };
};

const rounded = (way: readonly Point[]) => {
  const corners = way.slice(1, -1).map((corner, at) => {
    const [before, after] = [way[at], way[at + 2]];
    const ease = Math.min(
      6,
      Math.hypot(corner.x - before.x, corner.y - before.y) / 2,
      Math.hypot(after.x - corner.x, after.y - corner.y) / 2,
    );
    const into = toward(corner, before, ease);
    const out = toward(corner, after, ease);
    return `L${into.x} ${into.y}Q${corner.x} ${corner.y} ${out.x} ${out.y}`;
  });
  const last = way[way.length - 1];
  return `M${way[0].x} ${way[0].y}${corners.join("")}L${last.x} ${last.y}`;
};

const lengthOf = (a: Point, b: Point) => Math.abs(b.x - a.x) + Math.abs(b.y - a.y);

const longestRun = (way: readonly Point[]): readonly [Point, Point] =>
  way
    .slice(2)
    .map((one, at) => [way[at + 1], one] as const)
    .reduce<readonly [Point, Point]>(
      (best, run) => (lengthOf(run[0], run[1]) > lengthOf(best[0], best[1]) ? run : best),
      [way[0], way[1]],
    );

/** An arrow in right-angled runs between the nearest sides, round any shape in its way; the window and the picture both draw this one. */
export const routeOf = (
  from: Rect,
  to: Rect,
  around: readonly Rect[] = [],
  within: Rect | null = null,
): Route => {
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
  const way =
    ways.find((one) => free(one, others)) ??
    detoursBetween(start, end, leaving, others, within).find((one) => free(one, others));
  if (!way) return { ...plain, way: ways[0] ?? [] };
  if (way === ways[0]) return { ...plain, way };
  if (way.length === 4) {
    const turn = { x: (way[1].x + way[2].x) / 2, y: (way[1].y + way[2].y) / 2 };
    const [drawn, x, y] = stepped(start, end, leaving, entering, turn);
    return { ...plain, path: drawn, labelX: x, labelY: y, way };
  }
  const [a, b] = longestRun(way);
  return {
    ...plain,
    path: rounded(way),
    labelX: (a.x + b.x) / 2,
    labelY: (a.y + b.y) / 2,
    way,
  };
};

const covered = (one: Rect, other: Rect, room: number) => {
  const wide =
    Math.min(one.x + one.wide, other.x + other.wide + room) - Math.max(one.x, other.x - room);
  const tall =
    Math.min(one.y + one.tall, other.y + other.tall + room) - Math.max(one.y, other.y - room);
  return wide > 0 && tall > 0 ? wide * tall : 0;
};

const END_WORD_TALL = 14;

export const endWordBox = (at: Point, side: Position, said: string): Rect => {
  const word = endWordAt(at, side);
  const wide = said.length * 6.2;
  const x = word.anchor === "end" ? word.x - wide : word.x;
  return { x, y: word.y - END_WORD_TALL / 2, wide, tall: END_WORD_TALL };
};

const spotsAlong = (route: Route, size: { wide: number; tall: number }): Point[] => {
  const runs = route.way.slice(1).map((one, at) => [route.way[at], one] as const);
  const along = [...runs].reverse().flatMap(([a, b]) =>
    [0.5, 0.25, 0.75].map((part) => ({
      x: a.x + (b.x - a.x) * part,
      y: a.y + (b.y - a.y) * part,
    })),
  );
  const beside = runs.flatMap(([a, b]) => {
    const middle = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
    const step = a.x === b.x ? { x: size.wide / 2 + 4, y: 0 } : { x: 0, y: size.tall / 2 + 4 };
    return [
      { x: middle.x + step.x, y: middle.y + step.y },
      { x: middle.x - step.x, y: middle.y - step.y },
    ];
  });
  return [{ x: route.labelX, y: route.labelY }, ...along, ...beside];
};

/** The room a drawing keeps round its shapes, which its arrows and their words stay inside. */
export const drawnWithin = (shapes: readonly Rect[]): Rect | null => {
  if (!shapes.length) return null;
  const left = Math.min(...shapes.map((one) => one.x)) - MARGIN;
  const top = Math.min(...shapes.map((one) => one.y)) - MARGIN;
  const right = Math.max(...shapes.map((one) => one.x + one.wide)) + MARGIN;
  const bottom = Math.max(...shapes.map((one) => one.y + one.tall)) + MARGIN;
  return { x: left, y: top, wide: right - left, tall: bottom - top };
};

const runsOf = (way: readonly Point[]) =>
  way
    .slice(1)
    .map((one, at) => runOf(way[at], one))
    .filter((run) => run.wide || run.tall);

export const sharedRun = (run: Rect, runs: readonly Rect[]) =>
  runs.some((own) =>
    run.wide === 0
      ? own.wide === 0 && own.x === run.x && own.y < run.y + run.tall && run.y < own.y + own.tall
      : own.tall === 0 && own.y === run.y && own.x < run.x + run.wide && run.x < own.x + own.wide,
  );

/** Every arrow of a drawing, each word where it covers no shape, no end word, no other arrow's word or line and stays in the drawing, or as near as can be. */
export const routesOf = (joints: readonly Joint[], shapes: readonly Rect[]): Route[] => {
  const within = drawnWithin(shapes);
  const routes = joints.map((joint) => routeOf(joint.from, joint.to, shapes, within));
  const lines = routes.map((route) => runsOf(route.way));
  const taken: Rect[] = joints.flatMap((joint, at) => {
    const route = routes[at];
    return [
      ...(joint.startLabel ? [endWordBox(route.from, route.fromSide, joint.startLabel)] : []),
      ...(joint.endLabel ? [endWordBox(route.to, route.toSide, joint.endLabel)] : []),
    ];
  });
  return routes.map((route, at) => {
    const said = joints[at].label;
    if (!said) return route;
    const size = pillSize(said);
    const box = (spot: Point): Rect => ({
      x: spot.x - size.wide / 2,
      y: spot.y - size.tall / 2,
      wide: size.wide,
      tall: size.tall,
    });
    const outside = (spot: Point) =>
      within ? size.wide * size.tall - covered(box(spot), within, 0) : 0;
    const crossed = lines
      .flatMap((runs, other) => (other === at ? [] : runs))
      .filter((run) => !sharedRun(run, lines[at]));
    const crossing = (spot: Point) =>
      crossed.reduce((sum, run) => sum + covered(box(spot), run, 1), 0);
    // A line under a word reads as cut, while a word past the margin only makes the picture grow.
    const cost = (spot: Point) =>
      shapes.reduce((sum, one) => sum + covered(box(spot), one, 2), 0) +
      taken.reduce((sum, one) => sum + covered(box(spot), one, 4), 0) +
      crossing(spot) * 40 +
      outside(spot) / 4;
    const spots = spotsAlong(route, size);
    const spot =
      spots.find((one) => cost(one) === 0) ??
      spots.slice(1).reduce((best, one) => (cost(one) < cost(best) ? one : best), spots[0]);
    taken.push(box(spot));
    return { ...route, labelX: spot.x, labelY: spot.y };
  });
};
