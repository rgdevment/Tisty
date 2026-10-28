import type { NodeChange } from "@xyflow/react";
import { useMemo, useRef, useState } from "react";
import type { Flow } from "./flow";
import {
  type FlowAltered,
  type FlowPicked,
  nodesPlaced,
  pinned,
  reread,
  type Spot,
} from "./flowing";
import { laneMoved, laneResized, regrouped } from "./flowlanes";
import type { Rect } from "./graphlayout";
import type { Lane } from "./scenes";
import type { Flowing } from "./useFlow";

// A group's node is named apart from the shapes, whose ids are the diagram's own.
export const GROUP = "group:";

type Point = { readonly x: number; readonly y: number };

export const settledInto = (flow: Flow, picked: FlowPicked): FlowAltered => ({
  body: flow.body,
  places: flow.places,
  picked,
});

export const chosenOf = (picked: FlowPicked): readonly string[] => {
  if (picked?.kind === "node") return [picked.id];
  return picked?.kind === "nodes" ? picked.ids : [];
};

/** Shapes and lanes as they are dragged and sized, written into the text once let go. */
export const useCanvasMoves = (
  editing: Flowing,
  still: ReadonlyMap<string, Rect>,
  lanes: ReadonlyMap<string, Lane>,
) => {
  const { flow, picked, change, pick } = editing;
  // While a shape or a lane moves it is drawn from here; letting go writes it into the text.
  const [moving, setMoving] = useState<ReadonlyMap<string, Spot>>(new Map());
  const resizing = useRef(false);
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
    const group = flow.nodes.find((one) => one.id === id)?.group;
    return group ? lanes.get(group)?.rect : undefined;
  };
  const inside = (id: string, at: Point): Point => {
    const lane = laneOf(id);
    return lane ? { x: at.x - lane.x, y: at.y - lane.y } : at;
  };

  // Lanes first, then the shapes dropped in them, which may join or leave a lane on the way.
  const settle = (spots: ReadonlyMap<string, Spot>) => {
    let now = pinned(flow, laid);
    const shifted: { id: string; x: number; y: number }[] = [];
    for (const [key, at] of spots) {
      if (key.startsWith(GROUP)) now = reread(now, laneMoved(now, key.slice(GROUP.length), at));
      else shifted.push({ id: key, x: at.x, y: at.y });
    }
    if (shifted.length) {
      now = reread(now, nodesPlaced(now, shifted, null));
      const rects = shifted.flatMap((one) => {
        const rect = laid.get(one.id);
        return rect ? [[one.id, { ...rect, x: one.x, y: one.y }] as const] : [];
      });
      now = regrouped(now, new Map(rects));
    }
    const first = [...spots.keys()][0];
    if (picked?.kind === "nodes") change(settledInto(now, picked));
    else if (first.startsWith(GROUP))
      change(settledInto(now, { kind: "group", id: first.slice(GROUP.length) }));
    else change(settledInto(now, { kind: "node", id: first }));
  };
  // Several shapes chosen by a box or by Ctrl-click are chosen together; one is chosen by its click.
  const reselected = (changes: NodeChange[]) => {
    const now = new Set(chosenOf(picked));
    for (const one of changes)
      if (one.type === "select" && !one.id.startsWith(GROUP))
        one.selected ? now.add(one.id) : now.delete(one.id);
    if (now.size > 1) pick({ kind: "nodes", ids: [...now] });
  };
  // A drag, a nudge with the arrow keys or a lane's new size is written once it ends.
  const moved = (changes: NodeChange[]) => {
    reselected(changes);
    const spots = new Map(moving);
    let touched = false;
    let ended = false;
    for (const one of changes) {
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
    settle(spots);
  };
  const sizing = () => {
    resizing.current = true;
  };
  const sized = (lane: string, box: Rect) => {
    resizing.current = false;
    setMoving(new Map());
    change(laneResized(pinned(flow, laid), lane, box));
  };

  return { moving, laid, inside, moved, sizing, sized };
};
