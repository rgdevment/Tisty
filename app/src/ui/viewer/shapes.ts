export type Drawn = { tag: "path" | "rect" | "circle"; at: Record<string, string> };

const path = (d: string): Drawn => ({ tag: "path", at: { d } });
const dot = (cx: string): Drawn => ({ tag: "circle", at: { cx, cy: "8", r: "1.2" } });

export const SHAPES = {
  list: [path("M3 4.5h10M3 8h7M3 11.5h10")],
  reload: [path("M13.2 8.6A5.3 5.3 0 1 1 11.8 4"), path("M12.7 1.8v3h-3")],
  apart: [
    path("M9.5 2.5H13.5V6.5M13.5 2.5L8 8"),
    path("M6.5 3.5H3.5A1 1 0 0 0 2.5 4.5V12.5A1 1 0 0 0 3.5 13.5H11.5A1 1 0 0 0 12.5 12.5V9.5"),
  ],
  lock: [
    { tag: "rect", at: { x: "3.5", y: "7", width: "9", height: "6.5", rx: "1.5" } },
    path("M5.5 7V5.2a2.5 2.5 0 0 1 5 0V7"),
  ],
  doc: [
    path("M4 1.8h5l3.5 3.5v9a.9.9 0 0 1-.9.9H4a.9.9 0 0 1-.9-.9V2.7A.9.9 0 0 1 4 1.8Z"),
    path("M9 1.8v3.6h3.5M5.5 8.5h5M5.5 11h3.5"),
  ],
  widget: [path("M8 2 14 8 8 14 2 8Z")],
  more: [dot("3.5"), dot("8"), dot("12.5")],
} satisfies Record<string, Drawn[]>;

export type Shaped = keyof typeof SHAPES;

// The same strokes drawn outside React, for the bars the editor builds by hand.
export const drawn = (name: Shaped, size: number): SVGSVGElement => {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  const at: Record<string, string> = {
    viewBox: "0 0 16 16",
    width: String(size),
    height: String(size),
    fill: name === "more" ? "currentColor" : "none",
    stroke: name === "more" ? "none" : "currentColor",
    "stroke-width": "1.6",
    "stroke-linecap": "round",
    "stroke-linejoin": "round",
    "aria-hidden": "true",
  };
  for (const [key, value] of Object.entries(at)) svg.setAttribute(key, value);
  for (const one of SHAPES[name]) {
    const part = document.createElementNS(ns, one.tag);
    for (const [key, value] of Object.entries(one.at)) part.setAttribute(key, value);
    svg.append(part);
  }
  return svg;
};
