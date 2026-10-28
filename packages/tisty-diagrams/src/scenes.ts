import type { ClassBox, Classes } from "./classes";
import type { Flow } from "./flow";
import { sizeOf } from "./flowoutlines";
import type { Size } from "./framing";
import { around, type Laid, layered, type Rect } from "./graphlayout";
import type { Place } from "./places";

// The board and the picture draw from these, so what is exported is what the window shows.
export interface Joint {
  readonly key: string;
  readonly from: string;
  readonly to: string;
  readonly label: string;
  readonly dashed: boolean;
  readonly thick: boolean;
  readonly start: Head | null;
  readonly end: Head | null;
  // A class relation's cardinalities, by the ends they belong to.
  readonly startLabel?: string;
  readonly endLabel?: string;
}

/** How an end of a line is drawn: an arrowhead, an open one, a hollow triangle or a diamond. */
export type Head = "arrow" | "open" | "triangle" | "diamond" | "solidDiamond";

export interface Group {
  readonly key: string;
  readonly rect: Rect;
  readonly title: string;
}

export interface Scene {
  readonly laid: Laid;
  readonly joints: readonly Joint[];
  readonly groups: readonly Group[];
}

const MARGIN = 40;

// Each one where it was placed, or laid out when it never was; a diagram placed whole needs no laying out.
const placedOver = (
  ids: readonly string[],
  sizes: ReadonlyMap<string, Size>,
  places: readonly Place[],
  lay: () => Laid,
): ReadonlyMap<string, Rect> => {
  const at = new Map(places.map((one) => [one.id, one]));
  const auto = ids.every((id) => at.has(id)) ? null : lay();
  return new Map(
    ids.map((id) => {
      const place = at.get(id);
      const size = sizes.get(id) ?? { wide: 0, tall: 0 };
      const rect = place ? { x: place.x, y: place.y, ...size } : auto?.at.get(id);
      return [id, rect ?? { x: 0, y: 0, ...size }] as const;
    }),
  );
};

const flowSizes = (flow: Flow) =>
  new Map(flow.nodes.map((one) => [one.id, sizeOf(one.text, one.shape ?? "process")]));

const autoLaid = (flow: Flow): Laid =>
  layered(
    flow.nodes.map((one) => one.id),
    flow.edges.map((one) => [one.from, one.to] as const),
    flowSizes(flow),
    flow.direction,
  );

/** Each node where it was placed, or laid out when it never was, in the window's own coordinates. */
export const flowLaid = (flow: Flow): ReadonlyMap<string, Rect> =>
  placedOver(
    flow.nodes.map((one) => one.id),
    flowSizes(flow),
    flow.places,
    () => autoLaid(flow),
  );

/** A group drawn where it was put, at its size: a column when it is taller than wide, else a row. */
export interface Lane {
  readonly id: string;
  readonly title: string;
  readonly rect: Rect;
  readonly across: boolean;
}

/** The groups drawn as lanes: those whose place carries a size. */
export const lanesIn = (
  groups: readonly { readonly id: string; readonly title: string }[],
  places: readonly Place[],
): Lane[] =>
  groups.flatMap((group) => {
    const place = places.find((one) => one.id === group.id);
    if (place?.wide === undefined || place.tall === undefined) return [];
    const rect = { x: place.x, y: place.y, wide: place.wide, tall: place.tall };
    return [{ id: group.id, title: group.title, rect, across: rect.wide > rect.tall }];
  });

export const lanesOf = (flow: Flow): Lane[] => lanesIn(flow.groups, flow.places);

const holds = (rect: Rect, x: number, y: number) =>
  x >= rect.x && x <= rect.x + rect.wide && y >= rect.y && y <= rect.y + rect.tall;

/** The lane a shape's middle is in; the smallest wins, so a lane drawn inside another can be reached. */
export const laneHolding = (lanes: readonly Lane[], rect: Rect): string | null =>
  lanes
    .filter((one) => holds(one.rect, rect.x + rect.wide / 2, rect.y + rect.tall / 2))
    .sort((a, b) => a.rect.wide * a.rect.tall - b.rect.wide * b.rect.tall)[0]?.id ?? null;

// Places may be anywhere, even below zero; a picture starts at its margin, lanes and all.
const framed = (at: ReadonlyMap<string, Rect>, lanes: readonly Rect[]) => {
  const all = [...at.values(), ...lanes];
  const left = Math.min(...all.map((one) => one.x)) - MARGIN;
  const top = Math.min(...all.map((one) => one.y)) - MARGIN;
  const moved = (rect: Rect): Rect => ({ ...rect, x: rect.x - left, y: rect.y - top });
  const laid: Laid = {
    at: new Map([...at].map(([id, rect]) => [id, moved(rect)] as const)),
    size: {
      wide: Math.max(...all.map((one) => one.x + one.wide)) - left + MARGIN,
      tall: Math.max(...all.map((one) => one.y + one.tall)) - top + MARGIN,
    },
  };
  return { laid, moved };
};

/** Each edge as it is drawn: dotted, thick, with or without a head. */
export const jointsOf = (flow: Flow): Joint[] =>
  flow.edges.map((one, index) => ({
    key: String(index),
    from: one.from,
    to: one.to,
    label: one.label,
    dashed: one.arrow.includes("."),
    thick: one.arrow.includes("="),
    start: null,
    end: /[>ox]$/.test(one.arrow) ? "arrow" : null,
  }));

// What the window and the picture draw of groups: a lane at its size, or a frame around its shapes.
const sceneWith = (
  groups: readonly { readonly id: string; readonly title: string }[],
  membersOf: (group: string) => readonly string[],
  places: readonly Place[],
  shaped: boolean,
  laidOut: () => ReadonlyMap<string, Rect>,
  auto: () => Laid,
  joints: readonly Joint[],
): Scene => {
  const lanes = new Map(lanesIn(groups, places).map((one) => [one.id, one.rect]));
  const { laid, moved } =
    places.length && (shaped || lanes.size)
      ? framed(laidOut(), [...lanes.values()])
      : { laid: auto(), moved: (rect: Rect) => rect };
  const drawn = groups.flatMap((group) => {
    const lane = lanes.get(group.id);
    const rect = lane
      ? moved(lane)
      : around(membersOf(group.id).flatMap((id) => laid.at.get(id) ?? []));
    return rect ? [{ key: group.id, rect, title: group.title }] : [];
  });
  return { laid, joints, groups: drawn };
};

export const flowScene = (flow: Flow): Scene =>
  sceneWith(
    flow.groups,
    (group) => flow.nodes.filter((one) => one.group === group).map((one) => one.id),
    flow.places,
    flow.nodes.length > 0,
    () => flowLaid(flow),
    () => autoLaid(flow),
    jointsOf(flow),
  );

export const CLASS_HEAD = 32;
export const CLASS_LINE = 20;

/** The size a class takes for its name, its kind and its members, the same in the window and the picture. */
export const classSize = (one: ClassBox): Size => ({
  wide: Math.max(
    150,
    ...[one.name, ...one.members.map((m) => m.text)].map((said) => said.length * 7.2 + 36),
  ),
  tall: CLASS_HEAD + Math.max(one.members.length, 1) * CLASS_LINE + 10 + (one.note ? 14 : 0),
});

// The head is drawn at the end Mermaid puts it: `<|--` and `<--` on the left class.
const headLeft = (arrow: string) => arrow.startsWith("<");
// The parent or the whole goes above: it is on the right in `--|>`, `..|>`, `--*` and `--o`.
const wholeRight = (arrow: string) => /(\|>|\*|o)$/.test(arrow);

// Each relation's ends, drawn from the class it leaves to the one it points at.
const CLASS_HEADS: Record<string, readonly [Head | null, Head | null]> = {
  "<|--": [null, "triangle"],
  "<|..": [null, "triangle"],
  "--|>": [null, "triangle"],
  "..|>": [null, "triangle"],
  "*--": ["solidDiamond", null],
  "--*": [null, "solidDiamond"],
  "o--": ["diamond", null],
  "--o": [null, "diamond"],
  "-->": [null, "open"],
  "<--": [null, "open"],
  "..>": [null, "open"],
  "<..": [null, "open"],
};

const classAuto = (diagram: Classes): Laid =>
  layered(
    diagram.classes.map((one) => one.name),
    diagram.links.map((one): readonly [string, string] =>
      wholeRight(one.arrow) ? [one.right, one.left] : [one.left, one.right],
    ),
    new Map(diagram.classes.map((one) => [one.name, classSize(one)])),
    diagram.direction,
  );

/** Each class where it was placed, or laid out when it never was, in the window's own coordinates. */
export const classLaid = (diagram: Classes): ReadonlyMap<string, Rect> =>
  placedOver(
    diagram.classes.map((one) => one.name),
    new Map(diagram.classes.map((one) => [one.name, classSize(one)])),
    diagram.places,
    () => classAuto(diagram),
  );

/** Each relation as it is drawn: dashed or not, its two heads and its cardinalities at their ends. */
export const classJointsOf = (diagram: Classes): Joint[] =>
  diagram.links.map((one, index) => {
    const back = headLeft(one.arrow);
    const [start, end] = CLASS_HEADS[one.arrow] ?? [null, null];
    return {
      key: String(index),
      from: back ? one.right : one.left,
      to: back ? one.left : one.right,
      label: one.label,
      dashed: one.arrow.includes("."),
      thick: false,
      start,
      end,
      startLabel: back ? one.rightCard : one.leftCard,
      endLabel: back ? one.leftCard : one.rightCard,
    };
  });

export const classScene = (diagram: Classes): Scene =>
  sceneWith(
    diagram.groups.map((one) => ({ id: one.id, title: one.id })),
    (group) => diagram.classes.filter((one) => one.group === group).map((one) => one.name),
    diagram.places,
    diagram.classes.length > 0,
    () => classLaid(diagram),
    () => classAuto(diagram),
    classJointsOf(diagram),
  );
