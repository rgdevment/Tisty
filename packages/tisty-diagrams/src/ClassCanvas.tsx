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
import { ClassCard, type ClassNode, HeadMarkers } from "./ClassNodes";
import ClassToolbar from "./ClassToolbar";
import {
  classAdded,
  classesLinked,
  classesPinned,
  classPickedRemoved,
  classReread,
} from "./classing";
import { SORTS } from "./classkinds";
import { namespaceAdded, reclassed } from "./classspaces";
import { Wire, type WireEdge } from "./FlowWire";
import { useSay } from "./handing";
import Palette, { Sketch } from "./Palette";
import { settledInto } from "./places";
import { LaneBox, type LaneNode } from "./ShapeNodes";
import { useCanvasHands } from "./useCanvasHands";
import { type ClassNaming, useClassCanvas } from "./useClassCanvas";
import type { Classing } from "./useClasses";
import { GROUP } from "./useMoving";

const NODE_TYPES = { class: ClassCard, lane: LaneBox };
const EDGE_TYPES = { wire: Wire };
// A small diagram is shown at its own size, not blown up to fill the window.
const FITTED = { maxZoom: 1 };
// A class put down before it has members is about this big, so it lands centred where it is let go.
const NEW_CLASS = { wide: 150, tall: 62 };
// A card with its name above a line, and a band above that for its kind when it has one.
const CLASS_SKETCH = "M3,1 H31 V21 H3 Z M3,8 H31";
const KIND_SKETCH = "M3,1 H31 V21 H3 Z M3,10 H31 M11,4.5 H23";
// A package: a tab for its name, a card inside.
const SPACE_SKETCH = "M2,4 H32 V22 H2 Z M2,4 V1 H14 V4 M9,9 H25 V18 H9 Z";
const SPACE = { wide: 300, tall: 360 };

type Point = { readonly x: number; readonly y: number };
type Sort = "plain" | "interface" | "abstract" | "enumeration" | "namespace";

const SKETCHES: Partial<Record<Sort, string>> = { plain: CLASS_SKETCH, namespace: SPACE_SKETCH };

const noteOf = (sort: Sort) => (sort === "plain" || sort === "namespace" ? null : sort);

/** The class diagram to draw by hand: classes stay where they are left, relations follow them. */
export default function ClassCanvas({
  editing,
  dark,
}: Readonly<{ editing: Classing; dark: boolean }>) {
  return (
    <ReactFlowProvider>
      <Drawing editing={editing} dark={dark} />
    </ReactFlowProvider>
  );
}

function Drawing({ editing, dark }: Readonly<{ editing: Classing; dark: boolean }>) {
  const t = useSay();
  const { diagram, picked, change, pick, undo, redo } = editing;
  const { screenToFlowPosition } = useReactFlow();
  const [naming, setNaming] = useState<ClassNaming>(null);
  const { nodes, edges, laid, moved } = useClassCanvas(editing, naming, setNaming);
  const removal = useMemo(() => classPickedRemoved(diagram, picked), [diagram, picked]);
  const { pane, middleOfPane, droppedAt, keyed } = useCanvasHands({
    remove: removal && (() => change(removal)),
    undo,
    redo,
  });

  // A new class, of the kind asked for, centred where it is put, in the package it is put in, named at once.
  const added = (sort: Sort, middle: Point) => {
    const base = classesPinned(diagram, laid);
    if (sort === "namespace") {
      const box = { x: middle.x - SPACE.wide / 2, y: middle.y - SPACE.tall / 2, ...SPACE };
      const made = namespaceAdded(base, t("classNewNamespace"), t("classNew"), box);
      change(made);
      if (made.picked?.kind === "class") setNaming({ kind: "class", name: made.picked.name });
      return;
    }
    const at = { x: middle.x - NEW_CLASS.wide / 2, y: middle.y - NEW_CLASS.tall / 2 };
    const made = classAdded(base, t("classNew"), noteOf(sort), at);
    const name = made.picked?.kind === "class" ? made.picked.name : "";
    const now = reclassed(classReread(base, made), new Map([[name, { ...at, ...NEW_CLASS }]]));
    change(settledInto(now, made.picked));
    setNaming({ kind: "class", name });
  };
  const landed = (sort: Sort, at: Point) => {
    const middle = droppedAt(at);
    if (middle) added(sort, middle);
  };
  const joined = (from: string, to: string) => {
    // A dot let go back on its own class is a change of mind, not a relation to itself.
    if (from !== to) change(classesLinked(classesPinned(diagram, laid), from, to));
  };
  // A relation let go on a class joins it; let go where nothing is, it makes a new class there.
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
    const middle = screenToFlowPosition({ x: at.clientX, y: at.clientY });
    const base = classesPinned(diagram, laid);
    const where = { x: middle.x - NEW_CLASS.wide / 2, y: middle.y - NEW_CLASS.tall / 2 };
    const made = classAdded(base, t("classNew"), null, where);
    const name = made.picked?.kind === "class" ? made.picked.name : "";
    const now = classReread(base, made);
    change({ ...classesLinked(now, state.fromNode.id, name), places: now.places });
    setNaming({ kind: "class", name });
  };

  const sorts = SORTS.map(([note, word]) => ({
    key: (note ?? "plain") as Sort,
    label: t(word),
  }));

  return (
    <div className="flex min-h-0 min-w-0 flex-1">
      <Palette<Sort>
        groups={[
          { title: t("classPalette"), items: sorts },
          {
            title: t("classNamespaces"),
            items: [{ key: "namespace", label: t("classNamespace") }],
          },
        ]}
        sketch={(sort) => <Sketch paths={[SKETCHES[sort] ?? KIND_SKETCH]} />}
        onAdd={(sort) => added(sort, middleOfPane())}
        onDrop={landed}
      />
      {/* biome-ignore lint/a11y/noStaticElementInteractions: the keys belong to the canvas inside */}
      <div className="flex min-h-0 min-w-0 flex-1 flex-col" onKeyDown={keyed}>
        <ClassToolbar editing={editing} laid={laid} removal={removal} />
        <div ref={pane} className="relative min-h-0 flex-1 bg-desk">
          <HeadMarkers />
          <ReactFlow<ClassNode | LaneNode, WireEdge>
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
              if (node.type === "class") pick({ kind: "class", name: node.id });
              if (node.type === "lane")
                pick({ kind: "namespace", name: node.id.slice(GROUP.length) });
            }}
            onEdgeClick={(_, edge) => pick({ kind: "link", index: Number(edge.id) })}
            onEdgeDoubleClick={(_, edge) => setNaming({ kind: "link", index: Number(edge.id) })}
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
