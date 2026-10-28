import type { ClassBox, Classes } from "./classes";
import type { Flow } from "./flow";
import type { Shape } from "./flowshapes";
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

// Shapes CSS has no border for are cut out of a filled box instead.
export const CUTS: Partial<Record<Shape, string>> = {
  decision: "polygon(50% 0, 100% 50%, 50% 100%, 0 50%)",
  hexagon: "polygon(12% 0, 88% 0, 100% 50%, 88% 100%, 12% 100%, 0 50%)",
  lean: "polygon(12% 0, 100% 0, 88% 100%, 0 100%)",
  leanLeft: "polygon(0 0, 88% 0, 100% 100%, 12% 100%)",
  trapezoid: "polygon(12% 0, 88% 0, 100% 100%, 0 100%)",
  trapezoidDown: "polygon(0 0, 100% 0, 88% 100%, 12% 100%)",
  flag: "polygon(0 0, 100% 0, 100% 100%, 0 100%, 12% 50%)",
};

const flowSize = (text: string, shape: Shape) => {
  const wide = Math.max(90, Math.round(text.length * 7.5 + 36));
  if (shape === "circle" || shape === "double")
    return { wide: Math.max(wide, 72), tall: Math.max(wide, 72) };
  if (shape === "decision") return { wide: wide + 40, tall: 64 };
  return { wide, tall: 42 };
};

export const flowScene = (flow: Flow): Scene => {
  const sizes = new Map(
    flow.nodes.map((one) => [one.id, flowSize(one.text, one.shape ?? "process")]),
  );
  const laid = layered(
    flow.nodes.map((one) => one.id),
    flow.edges.map((one) => [one.from, one.to] as const),
    sizes,
    flow.direction,
  );
  const groups = flow.groups.flatMap((group) => {
    const rect = around(
      flow.nodes
        .filter((one) => one.group === group.id)
        .flatMap((one) => laid.at.get(one.id) ?? []),
    );
    return rect ? [{ key: group.id, rect, title: group.title }] : [];
  });
  const joints = flow.edges.map((one, index) => ({
    key: String(index),
    from: one.from,
    to: one.to,
    label: one.label,
    dashed: one.arrow.includes("."),
    thick: one.arrow.includes("="),
    arrow: /[>ox]$/.test(one.arrow),
  }));
  return { laid, joints, groups };
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
