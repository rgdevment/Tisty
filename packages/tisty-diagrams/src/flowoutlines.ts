import type { Shape } from "./flowshapes";
import type { Size } from "./framing";

/** A shape drawn in its own box from (0, 0): a body to fill and outline, and lines over it. */
export interface Outline {
  // Empty for shapes that are only lines or only words.
  readonly body: string;
  readonly lines: readonly string[];
  // How far below the middle its words sit, for shapes whose top is not for words.
  readonly drop: number;
}

const n = (value: number) => String(Math.round(value * 10) / 10);

const box = (x: number, y: number, wide: number, tall: number, radius: number) => {
  const r = Math.min(radius, wide / 2, tall / 2);
  if (r <= 0) return `M${n(x)},${n(y)} H${n(x + wide)} V${n(y + tall)} H${n(x)} Z`;
  const arc = (dx: number, dy: number) => `a${n(r)},${n(r)} 0 0 1 ${n(dx)},${n(dy)}`;
  return [
    `M${n(x + r)},${n(y)} H${n(x + wide - r)} ${arc(r, r)}`,
    `V${n(y + tall - r)} ${arc(-r, r)}`,
    `H${n(x + r)} ${arc(-r, -r)}`,
    `V${n(y + r)} ${arc(r, -r)} Z`,
  ].join(" ");
};

const ellipse = (x: number, y: number, wide: number, tall: number) => {
  const [rx, ry] = [n(wide / 2), n(tall / 2)];
  const middle = n(y + tall / 2);
  return `M${n(x)},${middle} A${rx},${ry} 0 1 1 ${n(x + wide)},${middle} A${rx},${ry} 0 1 1 ${n(x)},${middle} Z`;
};

const points = (...xy: readonly (readonly [number, number])[]) =>
  `M${xy.map(([x, y]) => `${n(x)},${n(y)}`).join(" L")} Z`;

// A page whose bottom edge waves, as a document is drawn.
const page = (x: number, y: number, wide: number, tall: number) => {
  const low = y + tall * 0.86;
  return `M${n(x)},${n(y)} H${n(x + wide)} V${n(low)} Q${n(x + wide * 0.75)},${n(low - tall * 0.16)} ${n(x + wide / 2)},${n(low)} T${n(x)},${n(low)} Z`;
};

const brace = (x: number, tall: number, side: 1 | -1) => {
  const tip = x - 6 * side;
  const mid = tall / 2;
  return `M${n(x + 6 * side)},0 Q${n(x)},0 ${n(x)},8 V${n(mid - 6)} Q${n(x)},${n(mid)} ${n(tip)},${n(mid)} Q${n(x)},${n(mid)} ${n(x)},${n(mid + 6)} V${n(tall - 8)} Q${n(x)},${n(tall)} ${n(x + 6 * side)},${n(tall)}`;
};

const plain = (body: string, ...lines: string[]): Outline => ({ body, lines, drop: 0 });

/** How a shape is drawn at a size; one Mermaid draws that the window does not know yet is a rounded box. */
export const outlineOf = (shape: Shape, wide: number, tall: number): Outline => {
  const slant = Math.min(tall / 2, wide * 0.12);
  const ry = Math.min(tall * 0.16, 10);
  switch (shape) {
    case "process":
      return plain(box(0, 0, wide, tall, 2));
    case "rounded":
      return plain(box(0, 0, wide, tall, 12));
    case "stadium":
      return plain(box(0, 0, wide, tall, tall / 2));
    case "subroutine":
      return plain(box(0, 0, wide, tall, 2), `M6,0 V${n(tall)}`, `M${n(wide - 6)},0 V${n(tall)}`);
    case "database":
      return {
        body: `M0,${n(ry)} A${n(wide / 2)},${n(ry)} 0 0 1 ${n(wide)},${n(ry)} V${n(tall - ry)} A${n(wide / 2)},${n(ry)} 0 0 1 0,${n(tall - ry)} Z`,
        lines: [`M0,${n(ry)} A${n(wide / 2)},${n(ry)} 0 0 0 ${n(wide)},${n(ry)}`],
        drop: ry / 2,
      };
    case "circle":
    case "sm-circ":
    case "f-circ":
      return plain(ellipse(0, 0, wide, tall));
    case "double":
    case "fr-circ":
      return plain(ellipse(0, 0, wide, tall), ellipse(4, 4, wide - 8, tall - 8));
    case "decision":
      return plain(points([wide / 2, 0], [wide, tall / 2], [wide / 2, tall], [0, tall / 2]));
    case "hexagon":
      return plain(
        points(
          [slant, 0],
          [wide - slant, 0],
          [wide, tall / 2],
          [wide - slant, tall],
          [slant, tall],
          [0, tall / 2],
        ),
      );
    case "lean":
      return plain(points([slant, 0], [wide, 0], [wide - slant, tall], [0, tall]));
    case "leanLeft":
      return plain(points([0, 0], [wide - slant, 0], [wide, tall], [slant, tall]));
    case "trapezoid":
      return plain(points([slant, 0], [wide - slant, 0], [wide, tall], [0, tall]));
    case "trapezoidDown":
      return plain(points([0, 0], [wide, 0], [wide - slant, tall], [slant, tall]));
    case "flag":
      return plain(points([0, 0], [wide, 0], [wide, tall], [0, tall], [slant, tall / 2]));
    case "person": {
      const head = tall * 0.17;
      const top = head * 2 + 4;
      return {
        body: box(0, top, wide, tall - top, 10),
        lines: [ellipse(wide / 2 - head, 0, head * 2, head * 2)],
        drop: top / 2,
      };
    }
    case "cloud":
      return plain(
        [
          `M${n(wide * 0.22)},${n(tall * 0.92)}`,
          `A${n(wide * 0.13)},${n(tall * 0.22)} 0 0 1 ${n(wide * 0.1)},${n(tall * 0.5)}`,
          `A${n(wide * 0.16)},${n(tall * 0.26)} 0 0 1 ${n(wide * 0.32)},${n(tall * 0.16)}`,
          `A${n(wide * 0.2)},${n(tall * 0.3)} 0 0 1 ${n(wide * 0.66)},${n(tall * 0.12)}`,
          `A${n(wide * 0.16)},${n(tall * 0.26)} 0 0 1 ${n(wide * 0.9)},${n(tall * 0.42)}`,
          `A${n(wide * 0.13)},${n(tall * 0.24)} 0 0 1 ${n(wide * 0.8)},${n(tall * 0.92)} Z`,
        ].join(" "),
      );
    case "doc":
      return { ...plain(page(0, 0, wide, tall)), drop: -tall * 0.05 };
    case "docs":
      return {
        body: page(0, 8, wide - 8, tall - 8),
        lines: [
          `M4,8 V4 H${n(wide - 4)} V${n(tall * 0.78)}`,
          `M8,4 V0 H${n(wide)} V${n(tall * 0.72)}`,
        ],
        drop: 0,
      };
    case "datastore":
      return plain(box(0, 0, wide, tall, 0), `M0,6 H${n(wide)}`, `M0,${n(tall - 6)} H${n(wide)}`);
    case "browser":
      return { ...plain(box(0, 0, wide, tall, 4), `M0,12 H${n(wide)}`), drop: 6 };
    case "console":
      return {
        // A window's top bar, its prompt in it, so the words below have the body to themselves.
        ...plain(box(0, 0, wide, tall, 4), `M0,12 H${n(wide)}`, "M6,3.5 l3,2.5 l-3,2.5 M11,8.5 h4"),
        drop: 6,
      };
    case "brace":
      return { body: "", lines: [brace(6, tall, 1)], drop: 0 };
    case "brace-r":
      return { body: "", lines: [brace(wide - 6, tall, -1)], drop: 0 };
    case "braces":
      return { body: "", lines: [brace(6, tall, 1), brace(wide - 6, tall, -1)], drop: 0 };
    case "text":
      return { body: "", lines: [], drop: 0 };
    default:
      return plain(box(0, 0, wide, tall, 8));
  }
};

const TALL: Partial<Record<Shape, number>> = {
  decision: 64,
  person: 84,
  cloud: 64,
  doc: 52,
  docs: 60,
  database: 56,
  browser: 54,
  console: 54,
};

/** The size a shape takes for its words, the same in the window and in the picture. */
export const sizeOf = (text: string, shape: Shape): Size => {
  const wide = Math.max(90, Math.round(text.length * 7.5 + 36));
  if (shape === "circle" || shape === "double" || shape === "fr-circ") {
    const side = Math.max(wide, 72);
    return { wide: side, tall: side };
  }
  if (shape === "sm-circ" || shape === "f-circ") return { wide: 28, tall: 28 };
  if (shape === "decision") return { wide: wide + 40, tall: 64 };
  if (shape === "cloud") return { wide: wide + 30, tall: 64 };
  return { wide, tall: TALL[shape] ?? 42 };
};
