import type { ClassBox, Classes } from "./classes";
import type { Flow } from "./flow";
import { sizeOf } from "./flowoutlines";
import { around, type Laid, layered, type Rect } from "./graphlayout";

// The board and the picture draw from these, so what is exported is what the window shows.
export interface Joint {
  readonly key: string;
  readonly from: string;
  readonly to: string;
  readonly label: string;
  readonly dashed: boolean;
  readonly thick: boolean;
  readonly arrow: boolean;
}

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

const autoLaid = (flow: Flow): Laid =>
  layered(
    flow.nodes.map((one) => one.id),
    flow.edges.map((one) => [one.from, one.to] as const),
    new Map(flow.nodes.map((one) => [one.id, sizeOf(one.text, one.shape ?? "process")])),
    flow.direction,
  );

/** Each node where it was placed, or laid out when it never was, in the window's own coordinates. */
export const flowLaid = (flow: Flow): ReadonlyMap<string, Rect> => {
  const places = new Map(flow.places.map((one) => [one.id, one]));
  // Laying out is the costly part, and a chart placed whole needs none of it.
  const auto = flow.nodes.every((one) => places.has(one.id)) ? null : autoLaid(flow);
  return new Map(
    flow.nodes.map((one) => {
      const place = places.get(one.id);
      const size = sizeOf(one.text, one.shape ?? "process");
      const rect = place ? { x: place.x, y: place.y, ...size } : auto?.at.get(one.id);
      return [one.id, rect ?? { x: 0, y: 0, ...size }] as const;
    }),
  );
};

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
    arrow: /[>ox]$/.test(one.arrow),
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

const classSize = (one: ClassBox) => ({
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

export const classScene = (diagram: Classes): Scene => {
  const laid = layered(
    diagram.classes.map((one) => one.name),
    diagram.links.map((one): readonly [string, string] =>
      wholeRight(one.arrow) ? [one.right, one.left] : [one.left, one.right],
    ),
    new Map(diagram.classes.map((one) => [one.name, classSize(one)])),
    diagram.direction,
  );
  const joints = diagram.links.map((one, index) => ({
    key: String(index),
    from: headLeft(one.arrow) ? one.right : one.left,
    to: headLeft(one.arrow) ? one.left : one.right,
    label: [one.label, one.arrow].filter(Boolean).join(" "),
    dashed: one.arrow.includes("."),
    thick: false,
    arrow: /[<>]/.test(one.arrow),
  }));
  return { laid, joints, groups: [] };
};
