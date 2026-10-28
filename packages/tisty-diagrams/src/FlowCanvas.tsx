import "@xyflow/react/dist/style.css";
import {
  Background,
  BackgroundVariant,
  ConnectionMode,
  type FinalConnectionState,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
} from "@xyflow/react";
import { useMemo, useState } from "react";
import FlowPalette, { type Piece } from "./FlowPalette";
import FlowToolbar from "./FlowToolbar";
import { Wire, type WireEdge } from "./FlowWire";
import type { Flow } from "./flow";
import { type FlowAltered, linked, nodeAdded, nodeAddedFrom, pinned, reread } from "./flowing";
import { LANE_SIZES, laneAdded, regrouped } from "./flowlanes";
import { sizeOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import { looksOf } from "./flowstyles";
import { nodeDuplicated, pickedRemoved } from "./flowtools";
import type { Rect } from "./graphlayout";
import { useSay } from "./handing";
import { shapeName } from "./paletteshapes";
import { LaneBox, type LaneNode, ShapeBox, type ShapeNode } from "./ShapeNodes";
import { flowLaid, flowLanesAround } from "./scenes";
import { useCanvasHands } from "./useCanvasHands";
import { GROUP, settledInto, useCanvasMoves } from "./useCanvasMoves";
import { type Naming, useCanvasNodes } from "./useCanvasNodes";
import type { Flowing } from "./useFlow";

const NODE_TYPES = { shape: ShapeBox, lane: LaneBox };
const EDGE_TYPES = { wire: Wire };
// A small diagram is shown at its own size, not blown up to fill the window.
const FITTED = { maxZoom: 1 };

type Point = { readonly x: number; readonly y: number };

const centered = (shape: Shape, text: string, middle: Point): Point => {
  const size = sizeOf(text, shape);
  return { x: middle.x - size.wide / 2, y: middle.y - size.tall / 2 };
};

/** The flowchart to draw by hand: shapes stay where they are left, arrows follow them, a palette beside. */
export default function FlowCanvas({
  editing,
  dark,
}: Readonly<{ editing: Flowing; dark: boolean }>) {
  return (
    <ReactFlowProvider>
      <Drawing editing={editing} dark={dark} />
    </ReactFlowProvider>
  );
}

function Drawing({ editing, dark }: Readonly<{ editing: Flowing; dark: boolean }>) {
  const t = useSay();
  const { flow, picked, change, pick, undo, redo } = editing;
  const { screenToFlowPosition } = useReactFlow();
  // The drawing alone, without the toolbar over it, is where pieces land.
  const [naming, setNaming] = useState<Naming>(null);
  const still = useMemo(() => flowLaid(flow), [flow]);
  const lanes = useMemo(
    () => new Map(flowLanesAround(flow, still).map((one) => [one.id, one])),
    [flow, still],
  );
  const looks = useMemo(() => looksOf(flow), [flow]);
  const { moving, laid, inside, moved, sizing, sized } = useCanvasMoves(editing, still, lanes);
  const { nodes, edges } = useCanvasNodes({
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
  });

  // A shape put down where a lane is becomes part of it, and its words are typed at once.
  const joinedIn = (base: Flow, made: FlowAltered, rect: Rect) => {
    const id = made.picked?.kind === "node" ? made.picked.id : "";
    change(settledInto(regrouped(reread(base, made), new Map([[id, rect]])), made.picked));
    setNaming({ kind: "node", id });
  };
  const put = (piece: Piece, middle: Point) => {
    const base = pinned(flow, laid);
    if (piece === "column" || piece === "row") {
      const size = LANE_SIZES[piece];
      const box = { x: middle.x - size.wide / 2, y: middle.y - size.tall / 2, ...size };
      change(laneAdded(base, t("flowNewLane"), box));
      return;
    }
    const text = shapeName(piece, t);
    const at = centered(piece, text, middle);
    joinedIn(base, nodeAdded(base, text, null, piece, at), { ...sizeOf(text, piece), ...at });
  };
  const landed = (piece: Piece, at: Point) => {
    const middle = droppedAt(at);
    if (middle) put(piece, middle);
  };

  const removal = useMemo(() => pickedRemoved(flow, picked), [flow, picked]);
  const duplicate =
    picked?.kind === "node"
      ? () => change(nodeDuplicated(pinned(flow, laid), picked.id, laid.get(picked.id)))
      : null;
  const { pane, middleOfPane, droppedAt, keyed } = useCanvasHands({
    remove: removal && (() => change(removal)),
    undo,
    redo,
    duplicate,
  });
  const joined = (from: string, to: string) => {
    // A dot let go back on its own shape is a change of mind, not a loop.
    if (from !== to) change(linked(pinned(flow, laid), from, to));
  };
  // An arrow let go on a shape joins it; let go where nothing is, it makes a new shape there.
  const ended = (e: MouseEvent | TouchEvent, state: FinalConnectionState) => {
    if (state.isValid || !state.fromNode) return;
    const at = "changedTouches" in e ? e.changedTouches[0] : e;
    const under = document
      .elementFromPoint(at.clientX, at.clientY)
      ?.closest<HTMLElement>("[data-node]")?.dataset.node;
    if (under) {
      joined(state.fromNode.id, under);
      return;
    }
    const text = t("flowNewNode");
    const where = centered("process", text, screenToFlowPosition({ x: at.clientX, y: at.clientY }));
    const base = pinned(flow, laid);
    const made = nodeAddedFrom(base, state.fromNode.id, text, where);
    joinedIn(base, made, { ...sizeOf(text, "process"), ...where });
  };

  return (
    <div className="flex min-h-0 min-w-0 flex-1">
      <FlowPalette onAdd={(piece) => put(piece, middleOfPane())} onDrop={landed} />
      {/* biome-ignore lint/a11y/noStaticElementInteractions: the keys belong to the canvas inside */}
      <div className="flex min-h-0 min-w-0 flex-1 flex-col" onKeyDown={keyed}>
        <FlowToolbar
          editing={editing}
          laid={laid}
          onRemove={() => change(removal)}
          removable={removal !== null}
          onDuplicate={duplicate}
        />
        <div ref={pane} className="relative min-h-0 flex-1 bg-desk">
          <ReactFlow<ShapeNode | LaneNode, WireEdge>
            nodes={nodes}
            edges={edges}
            nodeTypes={NODE_TYPES}
            edgeTypes={EDGE_TYPES}
            colorMode={dark ? "dark" : "light"}
            connectionMode={ConnectionMode.Loose}
            deleteKeyCode={null}
            zoomOnDoubleClick={false}
            minZoom={0.2}
            fitView
            fitViewOptions={FITTED}
            proOptions={{ hideAttribution: true }}
            onNodesChange={moved}
            onNodeClick={(e, node) => {
              // A click held with Ctrl or ⌘ adds to what is chosen, which React Flow has just told.
              if (e.ctrlKey || e.metaKey) return;
              if (node.type === "shape") pick({ kind: "node", id: node.id });
              if (node.type === "lane") pick({ kind: "group", id: node.id.slice(GROUP.length) });
            }}
            onEdgeClick={(_, edge) => pick({ kind: "edge", index: Number(edge.id) })}
            onEdgeDoubleClick={(_, edge) => setNaming({ kind: "edge", index: Number(edge.id) })}
            onPaneClick={() => pick(null)}
            onConnect={(made) => joined(made.source, made.target)}
            onConnectEnd={ended}
          >
            <Background variant={BackgroundVariant.Dots} gap={16} />
          </ReactFlow>
        </div>
      </div>
    </div>
  );
}
