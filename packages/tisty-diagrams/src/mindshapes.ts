import type { Shape } from "./mindmap";
import type { Word } from "./words";

/** What each shape of an idea is called, in the panel and the palette. */
export const SHAPE_WORDS: Record<Shape, Word> = {
  plain: "diagramShapePlain",
  square: "diagramShapeSquare",
  rounded: "diagramShapeRounded",
  circle: "diagramShapeCircle",
  cloud: "diagramShapeCloud",
  bang: "diagramShapeBang",
  hexagon: "diagramShapeHexagon",
};

/** The lines around an idea of each shape, on the map and in its sketches. */
export const IDEA_LINES: Record<Shape, string> = {
  plain: "rounded-md border border-transparent",
  square: "rounded-[3px] border",
  rounded: "rounded-lg border",
  circle: "rounded-full border",
  cloud: "rounded-full border border-dashed",
  bang: "rounded-sm border-2 border-dashed",
  hexagon: "rounded-md border",
};
