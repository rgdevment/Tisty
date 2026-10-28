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
import { type KeyboardEvent, useMemo, useRef, useState } from "react";
import FlowPalette from "./FlowPalette";
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
import { sideMiddle } from "./flowroutes";
import type { Shape } from "./flowshapes";
import { around } from "./graphlayout";
import { useHands, useSay } from "./handing";
import { shapeName } from "./paletteshapes";
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
  SIDES.map((side) => {
    const at = sideMiddle({ x: 0, y: 0, wide, tall }, side);
    return {
      id: side,
      type: "source" as const,
      position: side,
      x: at.x - HANDLE / 2,
      y: at.y - HANDLE / 2,
      width: HANDLE,
      height: HANDLE,
    };
  });

// Keys typed into a field, or pressed on the canvas's own buttons, are not for what is chosen.
const elsewhere = (e: KeyboardEvent) =>
  (e.target as Element).closest(
    "input, textarea, select, [contenteditable], .react-flow__panel",
  ) !== null;

const newlyPicked = (made: FlowAltered): Naming =>
  made.picked?.kind === "node" ? { kind: "node", id: made.picked.id } : null;

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
  const { button } = useHands().looks;
  const { flow, picked, change, pick } = editing;
  const { screenToFlowPosition } = useReactFlow();
  const room = useRef<HTMLDivElement>(null);
  const [naming, setNaming] = useState<Naming>(null);
  // While a shape is dragged it is drawn here; letting go writes it into the text.
  const [moving, setMoving] = useState<ReadonlyMap<string, Point>>(new Map());
  const still = useMemo(() => flowLaid(flow), [flow]);
  const laid = useMemo(() => {
    if (!moving.size) return still;
    const at = new Map(still);
    for (const [id, point] of moving) {
      const rect = at.get(id);
      if (rect) at.set(id, { ...rect, ...point });
    }
    return at;
  }, [still, moving]);

  const named = (made: FlowAltered | null, then: Naming = null) => {
    change(made);
    setNaming(then);
  };
  const added = (shape: Shape, middle: Point) => {
    const text = shapeName(shape, t);
    const made = nodeAdded(pinned(flow, laid), text, null, shape, centered(shape, text, middle));
    named(made, newlyPicked(made));
  };
  const middleOfRoom = () => {
    const box = room.current?.getBoundingClientRect();
    return screenToFlowPosition({
      x: (box?.left ?? 0) + (box?.width ?? 0) / 2,
      y: (box?.top ?? 0) + (box?.height ?? 0) / 2,
    });
  };
  const landed = (shape: Shape, at: Point) => {
    if (!room.current?.contains(document.elementFromPoint(at.x, at.y))) return;
    added(shape, screenToFlowPosition(at));
  };

  // The same objects while another shape is dragged, so only the one moving is drawn again.
  const placedShapes = useMemo(
    (): ShapeNode[] =>
      flow.nodes.flatMap((one) => {
        const rect = still.get(one.id);
        if (!rect) return [];
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
    [flow, still, picked, naming, t, pick, change],
  );
  const shapes = moving.size
    ? placedShapes.map((one) => {
        const point = moving.get(one.id);
        return point ? { ...one, position: point } : one;
      })
    : placedShapes;
  const frames = flow.groups.flatMap((group): FrameNode[] => {
    const rect = around(
      flow.nodes.filter((one) => one.group === group.id).flatMap((one) => laid.get(one.id) ?? []),
    );
    if (!rect) return [];
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
          picked: picked?.kind === "group" && picked.id === group.id,
          onPick: () => pick({ kind: "group", id: group.id }),
        },
      },
    ];
  });
  const wires = useMemo(
    (): WireEdge[] =>
      jointsOf(flow).map((one, index) => ({
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
          onNamed: (label) => {
            if (label !== null && label.trim() !== one.label)
              change(edgeChanged(flow, index, { label }));
            setNaming(null);
          },
        },
      })),
    [flow, picked, naming, t, pick, change],
  );

  // A drag, or a nudge with the arrow keys, is written once it ends, at the point it ended.
  const moved = (changes: NodeChange[]) => {
    const moves = changes.flatMap((one) =>
      one.type === "position" && one.position
        ? [{ id: one.id, at: one.position, dragging: one.dragging }]
        : [],
    );
    if (!moves.length) return;
    const next = new Map([...moving, ...moves.map((one) => [one.id, one.at] as const)]);
    if (moves.some((one) => one.dragging)) {
      setMoving(next);
      return;
    }
    setMoving(new Map());
    const placed = [...next].map(([id, at]) => ({ id, x: at.x, y: at.y }));
    change(nodesPlaced(pinned(flow, laid), placed, { kind: "node", id: moves[0].id }));
  };
  const remove = () => {
    if (picked?.kind === "node") change(nodeRemoved(flow, picked.id));
    if (picked?.kind === "edge") change(edgeRemoved(flow, picked.index));
    if (picked?.kind === "group") change(groupRemoved(flow, picked.id));
  };
  const keyed = (e: KeyboardEvent) => {
    if ((e.key !== "Delete" && e.key !== "Backspace") || elsewhere(e) || !picked) return;
    e.preventDefault();
    remove();
  };
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
    const middle = screenToFlowPosition({ x: at.clientX, y: at.clientY });
    const made = nodeAddedFrom(
      pinned(flow, laid),
      state.fromNode.id,
      text,
      centered("process", text, middle),
    );
    named(made, newlyPicked(made));
  };

  return (
    <div className="flex min-h-0 min-w-0 flex-1">
      <FlowPalette onAdd={(shape) => added(shape, middleOfRoom())} onDrop={landed} />
      {/* biome-ignore lint/a11y/noStaticElementInteractions: the keys belong to the canvas inside */}
      <div ref={room} className="relative min-h-0 min-w-0 flex-1 bg-desk" onKeyDown={keyed}>
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
          onNodeClick={(_, node) => {
            if (node.type === "shape") pick({ kind: "node", id: node.id });
          }}
          onEdgeClick={(_, edge) => pick({ kind: "edge", index: Number(edge.id) })}
          onEdgeDoubleClick={(_, edge) => setNaming({ kind: "edge", index: Number(edge.id) })}
          onPaneClick={() => pick(null)}
          onConnect={(made) => joined(made.source, made.target)}
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
