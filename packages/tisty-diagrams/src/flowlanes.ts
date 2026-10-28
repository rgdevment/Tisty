import type { Flow } from "./flow";
import { type FlowAltered, groupAdded, nodeMoved, placed, reread } from "./flowing";
import type { Rect } from "./graphlayout";
import type { Spot } from "./places";
import { lanesOf } from "./scenes";

/** The sizes a lane starts at: a column for steps that go down, a row for steps that go across. */
export const LANE_SIZES = {
  column: { wide: 260, tall: 480 },
  row: { wide: 640, tall: 160 },
} as const;

/** A group drawn as a lane where it was put, empty until shapes are let go in it. */
export const laneAdded = (flow: Flow, title: string, box: Rect): FlowAltered => {
  const made = groupAdded(flow, title);
  const id = made.picked?.kind === "group" ? made.picked.id : "";
  return { ...made, places: placed(flow, id, box) };
};

/** A lane moved with every shape in it, so they keep their places inside it. */
export const laneMoved = (flow: Flow, id: string, to: Spot): FlowAltered => {
  const lane = lanesOf(flow).find((one) => one.id === id);
  const picked = { kind: "group", id } as const;
  if (!lane) return { body: flow.body, picked };
  const dx = to.x - lane.rect.x;
  const dy = to.y - lane.rect.y;
  const shifted = flow.nodes
    .filter((one) => one.group === id)
    .reduce<Flow>((now, one) => {
      const place = now.places.find((at) => at.id === one.id);
      if (!place) return now;
      return { ...now, places: placed(now, one.id, { x: place.x + dx, y: place.y + dy }) };
    }, flow);
  return { body: flow.body, places: placed(shifted, id, to), picked };
};

/** A lane's new size, the shapes in it left where they are. */
export const laneResized = (flow: Flow, id: string, box: Rect): FlowAltered => ({
  body: flow.body,
  places: placed(flow, id, box),
  picked: { kind: "group", id },
});

const holds = (rect: Rect, x: number, y: number) =>
  x >= rect.x && x <= rect.x + rect.wide && y >= rect.y && y <= rect.y + rect.tall;

/** Shapes let go in a lane join it, outside every lane leave it; one its lane names in an arrow goes back. */
export const regrouped = (
  flow: Flow,
  at: ReadonlyMap<string, Rect>,
  was: ReadonlyMap<string, Rect> = new Map(),
): Flow => {
  const lanes = lanesOf(flow);
  let now = flow;
  for (const [id, rect] of at) {
    const node = now.nodes.find((one) => one.id === id);
    if (!node) continue;
    const x = rect.x + rect.wide / 2;
    const y = rect.y + rect.tall / 2;
    // The smallest lane holding it wins, so a lane drawn inside another can be reached.
    const into =
      lanes
        .filter((one) => holds(one.rect, x, y))
        .sort((a, b) => a.rect.wide * a.rect.tall - b.rect.wide * b.rect.tall)[0]?.id ?? null;
    const inLane = lanes.some((one) => one.id === node.group);
    if (into === node.group || (into === null && !inLane)) continue;
    const made = nodeMoved(now, id, into);
    const back = was.get(id);
    if (made) now = reread(now, made);
    else if (back) now = { ...now, places: placed(now, id, { x: back.x, y: back.y }) };
  }
  return now;
};
