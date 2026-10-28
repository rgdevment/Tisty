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

export const lanesOf = (flow: Flow): Lane[] =>
  flow.groups.flatMap((group) => {
    const place = flow.places.find((one) => one.id === group.id);
    if (place?.wide === undefined || place.tall === undefined) return [];
    const rect = { x: place.x, y: place.y, wide: place.wide, tall: place.tall };
    return [{ id: group.id, title: group.title, rect, across: rect.wide > rect.tall }];
  });

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

export const flowScene = (flow: Flow): Scene => {
  const lanes = new Map(lanesOf(flow).map((one) => [one.id, one.rect]));
  const placed = flow.places.length && (flow.nodes.length || lanes.size);
  const { laid, moved } = placed
    ? framed(flowLaid(flow), [...lanes.values()])
    : { laid: autoLaid(flow), moved: (rect: Rect) => rect };
  const groups = flow.groups.flatMap((group) => {
    const lane = lanes.get(group.id);
    const rect = lane
      ? moved(lane)
      : around(
          flow.nodes
            .filter((one) => one.group === group.id)
            .flatMap((one) => laid.at.get(one.id) ?? []),
        );
    return rect ? [{ key: group.id, rect, title: group.title }] : [];
  });
  return { laid, joints: jointsOf(flow), groups };
};

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

export const classScene = (diagram: Classes): Scene => {
  const laid =
    diagram.places.length && diagram.classes.length
      ? framed(classLaid(diagram), []).laid
      : classAuto(diagram);
  return { laid, joints: classJointsOf(diagram), groups: [] };
};
