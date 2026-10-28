import { nameOfShape, type Shape } from "./flowshapes";
import type { Say, Word } from "./words";

const SHAPE_WORDS: Partial<Record<Shape, Word>> = {
  process: "flowShapeProcess",
  rounded: "flowShapeRounded",
  stadium: "flowShapeStadium",
  circle: "flowShapeCircle",
  decision: "flowShapeDecision",
  hexagon: "flowShapeHexagon",
  database: "flowShapeDatabase",
  subroutine: "flowShapeSubroutine",
  lean: "flowShapeLean",
  flag: "flowShapeFlag",
  doc: "flowShapeDocument",
  docs: "flowShapeDocuments",
  datastore: "flowShapeDatastore",
  person: "flowShapePerson",
  cloud: "flowShapeCloud",
  browser: "flowShapeBrowser",
  console: "flowShapeConsole",
  brace: "flowShapeNote",
  text: "flowShapeText",
};

/** A shape by its name in the window, or by Mermaid's when the window has none for it. */
export const shapeName = (shape: Shape, t: Say): string => {
  const word = SHAPE_WORDS[shape];
  return word ? t(word) : nameOfShape(shape);
};

export const PALETTE: readonly { readonly title: Word; readonly shapes: readonly Shape[] }[] = [
  {
    title: "flowPaletteBasic",
    shapes: ["process", "stadium", "decision", "lean", "subroutine", "circle"],
  },
  { title: "flowPaletteData", shapes: ["database", "doc", "docs", "datastore"] },
  { title: "flowPalettePeople", shapes: ["person", "cloud", "browser", "console"] },
  { title: "flowPaletteNotes", shapes: ["brace", "text"] },
];

/** The shapes the panel offers: the palette's and the older bracketed ones. */
export const SHAPES: readonly Shape[] = [
  ...PALETTE.flatMap((one) => one.shapes),
  "rounded",
  "hexagon",
  "flag",
];
