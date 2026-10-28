import { getSmoothStepPath, Position } from "@xyflow/system";
import type { Rect } from "./graphlayout";

export interface Route {
  readonly path: string;
  readonly labelX: number;
  readonly labelY: number;
  // Where it leaves and where it arrives, for words written at its ends.
  readonly from: { readonly x: number; readonly y: number };
  readonly to: { readonly x: number; readonly y: number };
}

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

/** An arrow in right-angled runs between the nearest sides; the window and the picture both draw this one. */
export const routeOf = (from: Rect, to: Rect): Route => {
  const [leaving, entering] = sidesOf(from, to);
  const start = sideMiddle(from, leaving);
  const end = sideMiddle(to, entering);
  const [path, labelX, labelY] = getSmoothStepPath({
    sourceX: start.x,
    sourceY: start.y,
    sourcePosition: leaving,
    targetX: end.x,
    targetY: end.y,
    targetPosition: entering,
    borderRadius: 6,
    offset: 18,
  });
  return { path, labelX, labelY, from: start, to: end };
};
