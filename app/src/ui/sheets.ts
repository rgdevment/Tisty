import type { Paper } from "../core";

export const LEAVES: Paper[] = ["a4", "letter", "tabloid"];

const MILLIMETRES: Record<Paper, [number, number]> = {
  a4: [210, 297],
  letter: [215.9, 279.4],
  tabloid: [279.4, 431.8],
};

export const MARGIN_MM = { x: 20, y: 22 };

// A sheet on screen measures what it measures on paper, at 96 pixels to the inch.
const pixels = (mm: number) => Math.round((mm * 96) / 25.4);

const each = (side: 0 | 1) =>
  Object.fromEntries(LEAVES.map((one) => [one, pixels(MILLIMETRES[one][side])])) as Record<
    Paper,
    number
  >;

export const PAPER = each(0);

export const SHEET_TALL = each(1);

export const MARGIN = { x: pixels(MARGIN_MM.x), y: pixels(MARGIN_MM.y) };

export const PRINTED = `margin:${MARGIN_MM.y}mm ${MARGIN_MM.x}mm`;

export const PAGE: Record<Paper, string> = { a4: "A4", letter: "Letter", tabloid: "11in 17in" };

export const TALL = Object.fromEntries(
  LEAVES.map((one) => [one, MILLIMETRES[one][1] / MILLIMETRES[one][0]]),
) as Record<Paper, number>;

export const leafWord = (one: Paper) =>
  one === "a4" ? "leafA4" : one === "letter" ? "leafLetter" : "leafTabloid";

export const pageFor = (leaf: Paper) => {
  const id = "tisty-page";
  let tag = document.getElementById(id) as HTMLStyleElement | null;
  if (!tag) {
    tag = document.createElement("style");
    tag.id = id;
    document.head.append(tag);
  }
  tag.textContent = `@page { size: ${PAGE[leaf]}; ${PRINTED}; }`;
};
