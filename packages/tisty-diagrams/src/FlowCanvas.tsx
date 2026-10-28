import "@xyflow/react/dist/style.css";
import {
  Background,
  BackgroundVariant,
  ConnectionMode,
  Controls,
  type FinalConnectionState,
  MarkerType,
  type NodeChange,
  Panel,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
} from "@xyflow/react";
import { type DragEvent, type KeyboardEvent, useMemo, useRef, useState } from "react";
import FlowPalette, { DRAGGED } from "./FlowPalette";
import { Wire, type WireEdge } from "./FlowWire";
import {
  edgeChanged,
  edgeRemoved,
  type FlowAltered,
  groupAdded,
  groupRemoved,
  linked,
  nodeAdded,
  nodeAddedFrom,
  nodeChanged,
  nodeRemoved,
  nodesPlaced,
  pinned,
} from "./flowing";
import { sizeOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import { around } from "./graphlayout";
import { useHands, useSay } from "./handing";
import { SHAPES, shapeName } from "./paletteshapes";
import { FrameBox, type FrameNode, ShapeBox, type ShapeNode, SIDES } from "./ShapeNodes";
import { flowLaid, jointsOf } from "./scenes";
import type { Flowing } from "./useFlow";

const NODE_TYPES = { shape: ShapeBox, frame: FrameBox };
const EDGE_TYPES = { wire: Wire };
const HANDLE = 10;

type Naming = { kind: "node"; id: string } | { kind: "edge"; index: number } | null;
type Point = { readonly x: number; readonly y: number };

// Where each side's dot sits, so arrows are drawn before the browser has measured anything.
const handlesOf = (wide: number, tall: number) =>
  SIDES.map((side, index) => ({
    id: side,
    type: "source" as const,
    position: side,
    x: [wide / 2, wide, wide / 2, 0][index] - HANDLE / 2,
    y: [0, tall / 2, tall, tall / 2][index] - HANDLE / 2,
    width: HANDLE,
    height: HANDLE,
  }));

const typing = (e: KeyboardEvent) =>
  (e.target as Element).closest("input, textarea, select, [contenteditable]") !== null;

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
  const { button } = useHands().looks;
  const { flow, picked, change, pick } = editing;
  const { screenToFlowPosition } = useReactFlow();
  const room = useRef<HTMLDivElement>(null);
  const [naming, setNaming] = useState<Naming>(null);
  // While a shape is dragged it is drawn here; letting go writes it into the text.
  const [moving, setMoving] = useState<ReadonlyMap<string, Point>>(new Map());
  const laid = useMemo(() => {
    const at = new Map(flowLaid(flow));
    for (const [id, point] of moving) {
      const rect = at.get(id);
      if (rect) at.set(id, { ...rect, ...point });
    }
    return at;
  }, [flow, moving]);

  const named = (made: FlowAltered | null, then: Naming = null) => {
    change(made);
    setNaming(then);
  };
  const placedAt = (shape: Shape, text: string, middle: Point) => {
    const size = sizeOf(text, shape);
    return { x: middle.x - size.wide / 2, y: middle.y - size.tall / 2 };
  };
  const added = (shape: Shape, middle: Point) => {
    const text = shapeName(shape, t);
    const made = nodeAdded(pinned(flow, laid), text, null, shape, placedAt(shape, text, middle));
    named(made, made.picked?.kind === "node" ? { kind: "node", id: made.picked.id } : null);
  };
  const middleOfRoom = () => {
    const box = room.current?.getBoundingClientRect();
    return screenToFlowPosition({
      x: (box?.left ?? 0) + (box?.width ?? 0) / 2,
      y: (box?.top ?? 0) + (box?.height ?? 0) / 2,
    });
  };

  const shapes: ShapeNode[] = flow.nodes.flatMap((one) => {
    const rect = laid.get(one.id);
    if (!rect) return [];
    const isNaming = naming?.kind === "node" && naming.id === one.id;
    return [
      {
        id: one.id,
        type: "shape",
        position: { x: rect.x, y: rect.y },
        width: rect.wide,
        height: rect.tall,
        handles: handlesOf(rect.wide, rect.tall),
        selected: picked?.kind === "node" && picked.id === one.id,
        data: {
          text: one.text,
          shape: one.shape ?? "process",
          wide: rect.wide,
          tall: rect.tall,
          naming: isNaming,
          label: t("flowText"),
          onPick: () => pick({ kind: "node", id: one.id }),
          onName: () => setNaming({ kind: "node", id: one.id }),
          onNamed: (text) => named(text === null ? null : nodeChanged(flow, one.id, { text })),
        },
      },
    ];
  });
  const frames: FrameNode[] = flow.groups.flatMap((group) => {
    const rect = around(
      flow.nodes.filter((one) => one.group === group.id).flatMap((one) => laid.get(one.id) ?? []),
    );
    if (!rect) return [];
    const isPicked = picked?.kind === "group" && picked.id === group.id;
    return [
      {
        id: `group:${group.id}`,
        type: "frame",
        position: { x: rect.x, y: rect.y },
        width: rect.wide,
        height: rect.tall,
        zIndex: -1,
        draggable: false,
        selectable: false,
        data: {
          title: group.title,
          picked: isPicked,
          onPick: () => pick({ kind: "group", id: group.id }),
        },
      },
    ];
  });
  const wires: WireEdge[] = jointsOf(flow).map((one, index) => ({
    id: one.key,
    source: one.from,
    target: one.to,
    type: "wire",
    selected: picked?.kind === "edge" && picked.index === index,
    markerEnd: one.arrow
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
      onNamed: (label) => named(label === null ? null : edgeChanged(flow, index, { label })),
    },
  }));

  const moved = (changes: NodeChange[]) => {
    const moves = changes.flatMap((one) =>
      one.type === "position" && one.position ? [[one.id, one.position] as const] : [],
    );
    if (moves.length) setMoving((was) => new Map([...was, ...moves]));
  };
  const dropped = (id: string) => {
    const moves = [...moving].map(([one, at]) => ({ id: one, x: at.x, y: at.y }));
    setMoving(new Map());
    if (moves.length) change(nodesPlaced(pinned(flow, laid), moves, { kind: "node", id }));
  };
  const remove = () => {
    if (picked?.kind === "node") change(nodeRemoved(flow, picked.id));
    if (picked?.kind === "edge") change(edgeRemoved(flow, picked.index));
    if (picked?.kind === "group") change(groupRemoved(flow, picked.id));
  };
  const keyed = (e: KeyboardEvent) => {
    if ((e.key !== "Delete" && e.key !== "Backspace") || typing(e) || !picked) return;
    e.preventDefault();
    remove();
  };
  // An arrow let go on a shape joins it; let go where nothing is, it makes a new shape there.
  const ended = (e: MouseEvent | TouchEvent, state: FinalConnectionState) => {
    if (state.isValid || !state.fromNode) return;
    const at = "changedTouches" in e ? e.changedTouches[0] : e;
    const under = document
      .elementFromPoint(at.clientX, at.clientY)
      ?.closest<HTMLElement>("[data-node]")?.dataset.node;
    if (under) {
      change(linked(pinned(flow, laid), state.fromNode.id, under));
      return;
    }
    const text = t("flowNewNode");
    const where = placedAt("process", text, screenToFlowPosition({ x: at.clientX, y: at.clientY }));
    const made = nodeAddedFrom(pinned(flow, laid), state.fromNode.id, text, where);
    named(made, made.picked?.kind === "node" ? { kind: "node", id: made.picked.id } : null);
  };
  const landed = (e: DragEvent) => {
    const shape = SHAPES.find((one) => one === e.dataTransfer.getData(DRAGGED));
    if (!shape) return;
    e.preventDefault();
    added(shape, screenToFlowPosition({ x: e.clientX, y: e.clientY }));
  };

  return (
    <div className="flex min-h-0 min-w-0 flex-1">
      <FlowPalette onAdd={(shape) => added(shape, middleOfRoom())} />
      {/* biome-ignore lint/a11y/noStaticElementInteractions: the keys and drops belong to the canvas inside */}
      <div
        ref={room}
        className="relative min-h-0 min-w-0 flex-1 bg-desk"
        onKeyDown={keyed}
        onDragOver={(e) => {
          if (!e.dataTransfer.types.includes(DRAGGED)) return;
          e.preventDefault();
          e.dataTransfer.dropEffect = "copy";
        }}
        onDrop={landed}
      >
        <ReactFlow<ShapeNode | FrameNode, WireEdge>
          nodes={[...frames, ...shapes]}
          edges={wires}
          nodeTypes={NODE_TYPES}
          edgeTypes={EDGE_TYPES}
          colorMode={dark ? "dark" : "light"}
          connectionMode={ConnectionMode.Loose}
          deleteKeyCode={null}
          selectionOnDrag={false}
          zoomOnDoubleClick={false}
          minZoom={0.2}
          fitView
          proOptions={{ hideAttribution: true }}
          onNodesChange={moved}
          onNodeDragStop={(_, node) => dropped(node.id)}
          onNodeClick={(_, node) => {
            if (node.type === "shape") pick({ kind: "node", id: node.id });
          }}
          onEdgeClick={(_, edge) => pick({ kind: "edge", index: Number(edge.id) })}
          onPaneClick={() => pick(null)}
          onConnect={(made) => change(linked(pinned(flow, laid), made.source, made.target))}
          onConnectEnd={ended}
        >
          <Background variant={BackgroundVariant.Dots} gap={16} />
          <Controls showInteractive={false} />
          <Panel position="top-left" className="flex items-center gap-2">
            <button
              type="button"
              className={`${button} bg-panel`}
              onClick={() => change(groupAdded(flow, t("flowNewGroup")))}
            >
              {t("flowAddGroup")}
            </button>
            <p className="m-0 rounded-lg bg-panel px-3 py-1.5 text-[12px] text-faint shadow">
              {t("flowKeys")}
            </p>
          </Panel>
        </ReactFlow>
      </div>
    </div>
  );
}
