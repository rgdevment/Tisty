import type { NodeChange } from "@xyflow/react";
import { useCallback, useMemo, useRef, useState } from "react";
import { choiceAfter } from "./choosing";
import type { Rect } from "./graphlayout";
import type { Spot } from "./places";
import type { Lane } from "./scenes";

// A group's node is named apart from the shapes, whose ids are the diagram's own.
export const GROUP = "group:";

type Point = { readonly x: number; readonly y: number };

/** What a diagram does when its shapes and lanes are let go, chosen or sized. */
export interface MovingHands {
  readonly still: ReadonlyMap<string, Rect>;
  readonly lanes: ReadonlyMap<string, Lane>;
  readonly groupOf: (id: string) => string | null;
  readonly chosen: readonly string[];
  readonly choose: (choice: { several: readonly string[] } | { one: string }) => void;
  // Lanes are keyed `group:` and their id, shapes by their own.
  readonly settle: (spots: ReadonlyMap<string, Spot>, laid: ReadonlyMap<string, Rect>) => void;
  readonly resize: (lane: string, box: Rect, laid: ReadonlyMap<string, Rect>) => void;
}

/** Shapes and lanes as they are dragged and sized on a canvas, handed on once let go. */
export const useMoving = (hands: MovingHands) => {
  const { still, lanes, groupOf } = hands;
  // While a shape or a lane moves it is drawn from here; letting go writes it into the text.
  const [moving, setMoving] = useState<ReadonlyMap<string, Spot>>(new Map());
  const resizing = useRef(false);
  // A node rebuilt without `measured` is new to React Flow, which then drags it as if never drawn.
  const measured = useRef(new Map<string, { readonly width: number; readonly height: number }>());
  const laid = useMemo(() => {
    if (!moving.size) return still;
    const at = new Map(still);
    for (const [id, spot] of moving) {
      const rect = at.get(id);
      if (rect) at.set(id, { ...rect, x: spot.x, y: spot.y });
    }
    return at;
  }, [still, moving]);

  // A shape in a lane is placed inside it, so it moves with the lane while the lane is dragged.
  const laneOf = (id: string): Rect | undefined => {
    const group = groupOf(id);
    return group ? lanes.get(group)?.rect : undefined;
  };
  const inside = (id: string, at: Point): Point => {
    const lane = laneOf(id);
    return lane ? { x: at.x - lane.x, y: at.y - lane.y } : at;
  };

  // A drag, a nudge with the arrow keys or a lane's new size is written once it ends.
  const moved = (changes: NodeChange[]) => {
    const choice = choiceAfter(hands.chosen, changes, (id) => id.startsWith(GROUP));
    if (choice) hands.choose(choice);
    const spots = new Map(moving);
    let touched = false;
    let ended = false;
    for (const one of changes) {
      if (one.type === "dimensions" && one.dimensions) measured.current.set(one.id, one.dimensions);
      if (one.type === "position" && one.position) {
        const lane = one.id.startsWith(GROUP) ? undefined : laneOf(one.id);
        const at = lane ? { x: one.position.x + lane.x, y: one.position.y + lane.y } : one.position;
        spots.set(one.id, { ...spots.get(one.id), ...at });
        touched = true;
        ended ||= !one.dragging;
      }
      if (one.type === "dimensions" && one.dimensions && one.resizing) {
        const was = spots.get(one.id) ?? lanes.get(one.id.slice(GROUP.length))?.rect;
        spots.set(one.id, {
          x: was?.x ?? 0,
          y: was?.y ?? 0,
          wide: one.dimensions.width,
          tall: one.dimensions.height,
        });
        touched = true;
      }
    }
    if (!touched) return;
    if (resizing.current || !ended) {
      setMoving(spots);
      return;
    }
    setMoving(new Map());
    hands.settle(spots, laid);
  };
  const sizing = () => {
    resizing.current = true;
  };
  const sized = (lane: string, box: Rect) => {
    resizing.current = false;
    setMoving(new Map());
    hands.resize(lane, box, laid);
  };

  // A node's size as drawn, with what React Flow last measured of it, or the drawn size until it has.
  const measuredAs = useCallback(
    (id: string, wide: number, tall: number) => ({
      width: wide,
      height: tall,
      measured: measured.current.get(id) ?? { width: wide, height: tall },
    }),
    [],
  );

  return { moving, laid, inside, moved, sizing, sized, measuredAs };
};
