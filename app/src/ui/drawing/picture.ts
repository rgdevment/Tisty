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

/** A diagram drawn as plain SVG, ready to be written in either theme. */
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

export const ringed = (rect: Rect, look: Look): string =>
  `<ellipse cx="${at(rect.x + rect.wide / 2)}" cy="${at(rect.y + rect.tall / 2)}" rx="${at(rect.wide / 2)}" ry="${at(rect.tall / 2)}" ${stroked(look)}/>`;

/** A CSS `polygon(…)` in percents of a box, as SVG points over that box. */
export const cut = (polygon: string, rect: Rect, look: Look): string => {
  const points = [...polygon.matchAll(/([\d.]+)%?\s+([\d.]+)%?/g)].map(
    ([, x, y]) =>
      `${at(rect.x + (Number(x) / 100) * rect.wide)},${at(rect.y + (Number(y) / 100) * rect.tall)}`,
  );
  return `<polygon points="${points.join(" ")}" ${stroked(look)}/>`;
};

export const stroke = (d: string, color: string, width: number, dashed?: string, head = false) =>
  `<path d="${d}" fill="none" stroke="${color}" stroke-width="${width}" stroke-linecap="round"${dashed ? ` stroke-dasharray="${dashed}"` : ""}${head ? ' marker-end="url(#head)"' : ""}/>`;

export const straight = (a: { x: number; y: number }, b: { x: number; y: number }) =>
  `M ${at(a.x)} ${at(a.y)} L ${at(b.x)} ${at(b.y)}`;

/** A label on a line, in a pill the colour of the ground so the line does not run through it. */
export const pill = (x: number, y: number, text: string, palette: Palette, ringedWith?: string) => {
  const wide = text.length * 6.2 + 14;
  const rect = { x: x - wide / 2, y: y - 9, wide, tall: 18 };
  return `${boxed(rect, { fill: palette.desk, stroke: ringedWith, radius: 9 })}${writing({ x, y, text, size: 11, fill: palette.accent })}`;
};

/** The whole SVG file: its own size, the font asked for, and a ground only when one is wanted. */
export const svgOf = (picture: Picture, palette: Palette, ground: string | null, font: string) => {
  const { wide, tall } = picture.size;
  const head = `<defs><marker id="head" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="${palette.accent}"/></marker></defs>`;
  const back = ground ? `<rect width="100%" height="100%" fill="${ground}"/>` : "";
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${at(wide)}" height="${at(tall)}" viewBox="0 0 ${at(wide)} ${at(tall)}" font-family="${escaped(font)}">${head}${back}${picture.drawn(palette)}</svg>`;
};
