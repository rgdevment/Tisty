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
