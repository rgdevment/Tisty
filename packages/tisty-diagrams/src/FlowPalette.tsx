import { outlineOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import { useSay } from "./handing";
import Palette, { Sketch } from "./Palette";
import { PALETTE, shapeName } from "./paletteshapes";

type Point = { readonly x: number; readonly y: number };

/** What the palette puts on the canvas: a shape, or a lane running down or across. */
export type Piece = Shape | "column" | "row";

const LANE_SKETCHES = {
  column: "M2,1 H32 V21 H2 Z M2,6 H32 M12,1 V21 M22,1 V21",
  row: "M2,1 H32 V21 H2 Z M7,1 V21 M2,8 H32 M2,15 H32",
};
// Words have no outline; three lines stand for them.
const WORDS_SKETCH = "M8,7 H26 M8,12 H26 M8,17 H19";

const sketchOf = (piece: Piece) => {
  if (piece === "column" || piece === "row") return <Sketch paths={[LANE_SKETCHES[piece]]} />;
  const outline = outlineOf(piece, 34, 22);
  const paths = [outline.body, ...outline.lines].filter(Boolean);
  return <Sketch paths={paths.length ? paths : [WORDS_SKETCH]} />;
};

/** The flowchart's shapes and lanes, to drag onto the canvas or press into its middle. */
export default function FlowPalette({
  onAdd,
  onDrop,
}: Readonly<{ onAdd: (piece: Piece) => void; onDrop: (piece: Piece, at: Point) => void }>) {
  const t = useSay();
  const groups = [
    ...PALETTE.map((group) => ({
      title: t(group.title),
      items: group.shapes.map((shape) => ({ key: shape as Piece, label: shapeName(shape, t) })),
    })),
    {
      title: t("flowLanes"),
      items: [
        { key: "column" as const, label: t("flowLaneColumn") },
        { key: "row" as const, label: t("flowLaneRow") },
      ],
    },
  ];
  return (
    <Palette<Piece>
      groups={groups}
      sketch={sketchOf}
      hint={t("flowPaletteHint")}
      onAdd={onAdd}
      onDrop={onDrop}
    />
  );
}
