export type Paper = "a4" | "letter" | "folio" | "tabloid";

const MILLIMETRES: Record<Paper, [number, number]> = {
  a4: [210, 297],
  letter: [215.9, 279.4],
  folio: [215.9, 330.2],
  tabloid: [279.4, 431.8],
};

export const PAPERS = Object.keys(MILLIMETRES) as Paper[];

// The papers a reader picks from; one a document asks for beyond them is offered too.
const LEAVES: Paper[] = ["a4", "letter", "folio"];

export const offered = (leaf: Paper): Paper[] =>
  LEAVES.includes(leaf) ? LEAVES : [...LEAVES, leaf];

const MARGIN_MM = { x: 20, y: 22 };

// A sheet on screen measures what it measures on paper, at 96 pixels to the inch.
const pixels = (mm: number) => Math.round((mm * 96) / 25.4);

const each = (side: 0 | 1) =>
  Object.fromEntries(PAPERS.map((one) => [one, pixels(MILLIMETRES[one][side])])) as Record<
    Paper,
    number
  >;

export const PAPER = each(0);

const SHEET_TALL = each(1);

const MARGIN = { x: pixels(MARGIN_MM.x), y: pixels(MARGIN_MM.y) };

export const PRINTED = `margin:${MARGIN_MM.y}mm ${MARGIN_MM.x}mm`;

export const acrossOf = (leaf: Paper, landscape: boolean) => (landscape ? SHEET_TALL : PAPER)[leaf];

export const writtenAcrossOf = (leaf: Paper, landscape: boolean) =>
  acrossOf(leaf, landscape) - 2 * MARGIN.x;

export const sheetFor = (leaf: Paper, landscape: boolean, said: string) => ({
  wide: acrossOf(leaf, landscape),
  tall: (landscape ? PAPER : SHEET_TALL)[leaf],
  ...MARGIN,
  said,
});

const PAGE: Record<Paper, string> = {
  a4: "A4",
  letter: "Letter",
  folio: "8.5in 13in",
  tabloid: "11in 17in",
};

const ACROSS: Record<Paper, string> = {
  a4: "A4 landscape",
  letter: "Letter landscape",
  folio: "13in 8.5in",
  tabloid: "17in 11in",
};

export const sizeOf = (leaf: Paper, landscape = false) => (landscape ? ACROSS : PAGE)[leaf];

export const TALL = Object.fromEntries(
  PAPERS.map((one) => [one, MILLIMETRES[one][1] / MILLIMETRES[one][0]]),
) as Record<Paper, number>;
