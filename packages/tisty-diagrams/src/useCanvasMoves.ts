import { type FlowPicked, manual, nodesPlaced, pinned, reread } from "./flowing";
import { laneMoved, laneResized, regrouped } from "./flowlanes";
import type { Rect } from "./graphlayout";
import { settledInto } from "./places";
import type { Lane } from "./scenes";
import type { Flowing } from "./useFlow";
import { GROUP, useMoving } from "./useMoving";

export { settledInto } from "./places";
export { GROUP } from "./useMoving";

export const chosenOf = (picked: FlowPicked): readonly string[] => {
  if (picked?.kind === "node") return [picked.id];
  return picked?.kind === "nodes" ? picked.ids : [];
};

/** The flowchart's shapes and lanes as they are dragged and sized, written into the text once let go. */
export const useCanvasMoves = (
  editing: Flowing,
  still: ReadonlyMap<string, Rect>,
  lanes: ReadonlyMap<string, Lane>,
) => {
  const { flow, picked, change, pick } = editing;
  return useMoving({
    still,
    lanes,
    groupOf: (id) => flow.nodes.find((one) => one.id === id)?.group ?? null,
    chosen: chosenOf(picked),
    choose: (choice) =>
      pick(
        "several" in choice
          ? { kind: "nodes", ids: choice.several }
          : { kind: "node", id: choice.one },
      ),
    // Lanes first, then the shapes dropped in them, which may join or leave a lane on the way.
    // A shape moved by hand ends laying out by itself.
    settle: (spots, laid) => {
      let now = pinned({ ...flow, body: manual(flow.body) }, laid);
      const shifted: { id: string; x: number; y: number }[] = [];
      for (const [key, at] of spots) {
        if (key.startsWith(GROUP)) {
          const id = key.slice(GROUP.length);
          now = reread(now, laneMoved(now, id, at, lanes.get(id)?.rect));
        } else shifted.push({ id: key, x: at.x, y: at.y });
      }
      if (shifted.length) {
        now = reread(now, nodesPlaced(now, shifted, null));
        const rects = shifted.flatMap((one) => {
          const rect = laid.get(one.id);
          return rect ? [[one.id, { ...rect, x: one.x, y: one.y }] as const] : [];
        });
        now = regrouped(now, new Map(rects), still, [...lanes.values()]);
      }
      const first = [...spots.keys()][0];
      if (picked?.kind === "nodes") change(settledInto(now, picked));
      else if (first.startsWith(GROUP))
        change(settledInto(now, { kind: "group", id: first.slice(GROUP.length) }));
      else change(settledInto(now, { kind: "node", id: first }));
    },
    resize: (lane, box, laid) => change(laneResized(pinned(flow, laid), lane, box)),
  });
};
