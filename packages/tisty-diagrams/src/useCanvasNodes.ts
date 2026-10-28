import { MarkerType } from "@xyflow/react";
import { useMemo } from "react";
import type { WireEdge } from "./FlowWire";
import { edgeChanged, nodeChanged } from "./flowing";
import type { Look } from "./flowstyles";
import type { Rect } from "./graphlayout";
import { useSay } from "./handing";
import type { Spot } from "./places";
import type { LaneNode, ShapeNode } from "./ShapeNodes";
import { handlesOf } from "./ShapeNodes";
import type { Lane } from "./scenes";
import { jointsOf } from "./scenes";
import { chosenOf, GROUP } from "./useCanvasMoves";
import type { Flowing } from "./useFlow";

export type Naming = { kind: "node"; id: string } | { kind: "edge"; index: number } | null;

type Point = { readonly x: number; readonly y: number };

/** What React Flow draws: lanes behind, the shapes in and over them, and the arrows. */
export const useCanvasNodes = ({
  editing,
  still,
  lanes,
  looks,
  moving,
  inside,
  naming,
  setNaming,
  sizing,
  sized,
}: {
  editing: Flowing;
  still: ReadonlyMap<string, Rect>;
  lanes: ReadonlyMap<string, Lane>;
  looks: ReadonlyMap<string, Look>;
  moving: ReadonlyMap<string, Spot>;
  inside: (id: string, at: Point) => Point;
  naming: Naming;
  setNaming: (naming: Naming) => void;
  sizing: () => void;
  sized: (lane: string, box: Rect) => void;
}) => {
  const t = useSay();
  const { flow, picked, change, pick } = editing;
  const chosen = chosenOf(picked);
  const chosenKey = chosen.join("\n");

  // The same objects while another shape is dragged, so only the one moving is drawn again.
  // biome-ignore lint/correctness/useExhaustiveDependencies: `chosenKey` stands for `chosen`, `lanes` for `inside`
  const placedShapes = useMemo(
    (): ShapeNode[] =>
      flow.nodes.flatMap((one) => {
        const rect = still.get(one.id);
        if (!rect) return [];
        const lane = one.group ? lanes.get(one.group) : undefined;
        return [
          {
            id: one.id,
            type: "shape",
            position: inside(one.id, rect),
            parentId: lane ? `${GROUP}${lane.id}` : undefined,
            width: rect.wide,
            height: rect.tall,
            handles: handlesOf(rect.wide, rect.tall),
            selected: chosen.includes(one.id),
            data: {
              text: one.text,
              shape: one.shape ?? "process",
              wide: rect.wide,
              tall: rect.tall,
              look: looks.get(one.id),
              naming: naming?.kind === "node" && naming.id === one.id,
              label: t("flowText"),
              onPick: () => pick({ kind: "node", id: one.id }),
              onName: () => setNaming({ kind: "node", id: one.id }),
              onNamed: (text) => {
                if (text !== null && text.trim() !== one.text)
                  change(nodeChanged(flow, one.id, { text }));
                setNaming(null);
              },
            },
          },
        ];
      }),
    [flow, still, lanes, looks, chosenKey, naming, t, pick, change, setNaming],
  );
  const shapes = placedShapes.map((one) => {
    const spot = moving.get(one.id);
    return spot ? { ...one, position: inside(one.id, spot) } : one;
  });
  const laneNodes = [...lanes.values()].map((lane): LaneNode => {
    const rect = { ...lane.rect, ...moving.get(`${GROUP}${lane.id}`) };
    const isPicked = picked?.kind === "group" && picked.id === lane.id;
    return {
      id: `${GROUP}${lane.id}`,
      type: "lane",
      position: { x: rect.x, y: rect.y },
      width: rect.wide,
      height: rect.tall,
      zIndex: -1,
      selected: isPicked,
      data: {
        title: lane.title,
        across: rect.wide > rect.tall,
        picked: isPicked,
        onPick: () => pick({ kind: "group", id: lane.id }),
        onResizing: sizing,
        onResized: (box) => sized(lane.id, box),
      },
    };
  });
  const wires = useMemo(
    (): WireEdge[] =>
      jointsOf(flow).map((one, index) => ({
        id: one.key,
        source: one.from,
        target: one.to,
        type: "wire",
        selected: picked?.kind === "edge" && picked.index === index,
        markerEnd: one.end
          ? { type: MarkerType.ArrowClosed, color: "var(--color-accent)", width: 16, height: 16 }
          : undefined,
        data: {
          label: one.label,
          title: `${one.from} → ${one.to}`,
          dashed: one.dashed,
          thick: one.thick,
          naming: naming?.kind === "edge" && naming.index === index,
          field: t("flowLabel"),
          onPick: () => pick({ kind: "edge", index }),
          onName: () => setNaming({ kind: "edge", index }),
          onNamed: (label) => {
            if (label !== null && label.trim() !== one.label)
              change(edgeChanged(flow, index, { label }));
            setNaming(null);
          },
        },
      })),
    [flow, picked, naming, t, pick, change, setNaming],
  );
  // Parents before the shapes in them, as React Flow asks.
  return { nodes: [...laneNodes, ...shapes], edges: wires };
};
