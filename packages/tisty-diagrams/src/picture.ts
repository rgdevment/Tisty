import type { Size } from "./framing";
import type { Rect } from "./graphlayout";

export interface Palette {
  readonly sheet: string;
  readonly ink: string;
  readonly soft: string;
  readonly faint: string;
  readonly line: string;
  readonly hover: string;
  readonly panel: string;
  readonly desk: string;
  readonly accent: string;
  readonly accentSoft: string;
}

// The tokens of `index.css`: a picture is drawn in either theme, whichever the window is in.
export const PALETTES: Record<"light" | "dark", Palette> = {
  light: {
    sheet: "#ffffff",
    ink: "#1d1d1f",
    soft: "#57575c",
    faint: "#67676b",
    line: "rgba(0,0,0,0.16)",
    hover: "rgba(0,0,0,0.035)",
    panel: "#fbfbfd",
    desk: "#eceef1",
    accent: "#0060e3",
    accentSoft: "rgba(0,96,227,0.08)",
  },
  dark: {
    sheet: "#232326",
    ink: "#f2f2f7",
    soft: "#adadb4",
    faint: "#9a9aa1",
    line: "rgba(255,255,255,0.2)",
    hover: "rgba(255,255,255,0.045)",
    panel: "#202022",
    desk: "#151516",
    accent: "#439bff",
    accentSoft: "rgba(67,155,255,0.11)",
  },
};

export interface Picture {
  readonly size: Size;
  readonly drawn: (palette: Palette) => string;
}

const ESCAPES: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" };
export const escaped = (text: string): string => text.replace(/[&<>"]/g, (one) => ESCAPES[one]);

const at = (n: number) => String(Math.round(n * 10) / 10);

// SVG text has no ellipsis: what the window would cut short is cut here by the same measure.
export const fitted = (text: string, room: number, perChar: number): string => {
  const most = Math.max(1, Math.floor(room / perChar));
  return text.length <= most ? text : `${text.slice(0, Math.max(1, most - 1))}…`;
};

export const MONO = "ui-monospace, Menlo, Consolas, monospace";

export interface Writing {
  readonly x: number;
  readonly y: number;
  readonly text: string;
  readonly size: number;
  readonly fill: string;
  readonly bold?: boolean;
  readonly anchor?: "start" | "middle" | "end";
  readonly mono?: boolean;
}

export const writing = (one: Writing): string =>
  `<text x="${at(one.x)}" y="${at(one.y)}" font-size="${one.size}" fill="${one.fill}" text-anchor="${one.anchor ?? "middle"}" dominant-baseline="central"${one.bold ? ' font-weight="600"' : ""}${one.mono ? ` font-family="${MONO}"` : ""}>${escaped(one.text)}</text>`;

export interface Look {
  readonly fill: string;
  readonly stroke?: string;
  readonly width?: number;
  readonly radius?: number;
  readonly ry?: number;
  readonly dashed?: string;
}

const stroked = (look: Look) =>
  `fill="${look.fill}"${look.stroke ? ` stroke="${look.stroke}" stroke-width="${look.width ?? 1}"` : ""}${look.dashed ? ` stroke-dasharray="${look.dashed}"` : ""}`;

export const boxed = (rect: Rect, look: Look): string =>
  `<rect x="${at(rect.x)}" y="${at(rect.y)}" width="${at(rect.wide)}" height="${at(rect.tall)}" rx="${at(look.radius ?? 0)}" ry="${at(look.ry ?? look.radius ?? 0)}" ${stroked(look)}/>`;

// The window cuts shapes with CSS percents; the picture has to say the same points in SVG.
export const cut = (polygon: string, rect: Rect, look: Look): string => {
  const points = [...polygon.matchAll(/([\d.]+)%?\s+([\d.]+)%?/g)].map(
    ([, x, y]) =>
      `${at(rect.x + (Number(x) / 100) * rect.wide)},${at(rect.y + (Number(y) / 100) * rect.tall)}`,
  );
  return `<polygon points="${points.join(" ")}" ${stroked(look)}/>`;
};

/** The heads a line may end in, each a marker of the picture by the same name. */
export type Ends = { readonly start?: string | null; readonly end?: string | null };

export const stroke = (d: string, color: string, width: number, dashed?: string, ends: Ends = {}) =>
  `<path d="${d}" fill="none" stroke="${color}" stroke-width="${width}" stroke-linecap="round"${dashed ? ` stroke-dasharray="${dashed}"` : ""}${ends.start ? ` marker-start="url(#${ends.start})"` : ""}${ends.end ? ` marker-end="url(#${ends.end})"` : ""}/>`;

// A hollow head is filled with the sheet so the line does not show through it.
const markers = (palette: Palette) =>
  [
    ["arrow", "M 0 0 L 10 5 L 0 10 z", palette.accent, palette.accent],
    ["open", "M 0 0 L 10 5 L 0 10", "none", palette.accent],
    ["triangle", "M 0 0 L 10 5 L 0 10 z", palette.sheet, palette.accent],
    ["diamond", "M 0 5 L 5 0 L 10 5 L 5 10 z", palette.sheet, palette.accent],
    ["solidDiamond", "M 0 5 L 5 0 L 10 5 L 5 10 z", palette.accent, palette.accent],
  ]
    .map(
      ([id, d, fill, line]) =>
        `<marker id="${id}" viewBox="-1 -1 12 12" refX="10" refY="5" markerWidth="9" markerHeight="9" orient="auto-start-reverse"><path d="${d}" fill="${fill}" stroke="${line}" stroke-width="1"/></marker>`,
    )
    .join("");

export const straight = (a: { x: number; y: number }, b: { x: number; y: number }) =>
  `M ${at(a.x)} ${at(a.y)} L ${at(b.x)} ${at(b.y)}`;

// The pill takes the colour of the ground so the line does not run through the words.
export const pill = (x: number, y: number, text: string, palette: Palette, ringedWith?: string) => {
  const wide = text.length * 6.2 + 14;
  const rect = { x: x - wide / 2, y: y - 9, wide, tall: 18 };
  return `${boxed(rect, { fill: palette.desk, stroke: ringedWith, radius: 9 })}${writing({ x, y, text, size: 11, fill: palette.accent })}`;
};

export const svgOf = (picture: Picture, palette: Palette, ground: string | null, font: string) => {
  const { wide, tall } = picture.size;
  const head = `<defs>${markers(palette)}</defs>`;
  const back = ground ? `<rect width="100%" height="100%" fill="${ground}"/>` : "";
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${at(wide)}" height="${at(tall)}" viewBox="0 0 ${at(wide)} ${at(tall)}" font-family="${escaped(font)}">${head}${back}${picture.drawn(palette)}</svg>`;
};
