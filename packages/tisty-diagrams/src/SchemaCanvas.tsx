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
import { type KeyboardEvent, useMemo, useRef, useState } from "react";
import { Wire, type WireEdge } from "./FlowWire";
import { useSay } from "./handing";
import Palette, { Sketch } from "./Palette";
import { columnOfHandle, TableCard, type TableNode } from "./SchemaNodes";
import SchemaToolbar from "./SchemaToolbar";
import { tableAddedAt } from "./schemaplaces";
import { related, schemaPickedRemoved } from "./scheming";
import type { Scheming } from "./useSchema";
import { type SchemaNaming, useSchemaCanvas } from "./useSchemaCanvas";

const NODE_TYPES = { table: TableCard };
const EDGE_TYPES = { wire: Wire };
// A small schema is shown at its own size, not blown up to fill the window.
const FITTED = { maxZoom: 1 };
// A card with its name on top and its columns in rows below.
const TABLE_SKETCH = "M3,1 H31 V21 H3 Z M3,7 H31 M3,12 H31 M3,17 H31";

type Point = { readonly x: number; readonly y: number };

// Keys typed into a field, or pressed on the canvas's own buttons, are not for what is chosen.
const elsewhere = (e: KeyboardEvent) =>
  (e.target as Element).closest(
    "input, textarea, select, [contenteditable], [role=toolbar], .react-flow__panel",
  ) !== null;

/** The schema to draw by hand: tables stay where they are left, relations follow them. */
export default function SchemaCanvas({
  editing,
  dark,
}: Readonly<{ editing: Scheming; dark: boolean }>) {
  return (
    <ReactFlowProvider>
      <Drawing editing={editing} dark={dark} />
    </ReactFlowProvider>
  );
}

function Drawing({ editing, dark }: Readonly<{ editing: Scheming; dark: boolean }>) {
  const t = useSay();
  const { schema, picked, change, pick, undo, redo } = editing;
  const { screenToFlowPosition } = useReactFlow();
  const pane = useRef<HTMLDivElement>(null);
  const [naming, setNaming] = useState<SchemaNaming>(null);
  const { nodes, edges, laid, moved } = useSchemaCanvas(editing, naming, setNaming);
  const removal = useMemo(() => schemaPickedRemoved(schema, picked), [schema, picked]);

  // A new table with its id, centred where it is put, named at once.
  const added = (middle: Point) => {
    const made = tableAddedAt(schema, t("schemaNewTable"), middle);
    change(made);
    if (made.picked?.kind === "table") setNaming({ kind: "table", name: made.picked.name });
  };
  const middleOfPane = () => {
    const box = pane.current?.getBoundingClientRect();
    return screenToFlowPosition({
      x: (box?.left ?? 0) + (box?.width ?? 0) / 2,
      y: (box?.top ?? 0) + (box?.height ?? 0) / 2,
    });
  };
  const landed = (_: "table", at: Point) => {
    if (!pane.current?.contains(document.elementFromPoint(at.x, at.y))) return;
    added(screenToFlowPosition(at));
  };
  const keyed = (e: KeyboardEvent) => {
    if (elsewhere(e)) return;
    const mod = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    const act = (did: (() => void) | null | undefined) => {
      e.preventDefault();
      did?.();
    };
    if ((key === "delete" || key === "backspace") && removal) act(() => change(removal));
    else if (mod && key === "z") act(e.shiftKey ? redo : undo);
    else if (mod && key === "y") act(redo);
  };
  // A column's dot let go on another table relates them by that column.
  const joined = (table: string, handle: string | null | undefined, into: string) => {
    const index = columnOfHandle(handle);
    const made = index === null ? null : related(schema, { table, index }, into);
    if (made) change(made);
  };
  // Let go where no table is, the dot makes a new table there, related by that column.
  const ended = (e: MouseEvent | TouchEvent, state: FinalConnectionState) => {
    if (state.isValid || !state.fromNode) return;
    const index = columnOfHandle(state.fromHandle?.id);
    if (index === null) return;
    const at = "changedTouches" in e ? e.changedTouches[0] : e;
    const under = document
      .elementFromPoint(at.clientX, at.clientY)
      ?.closest<HTMLElement>("[data-node]")?.dataset.node;
    if (under) {
      joined(state.fromNode.id, state.fromHandle?.id, under);
      return;
    }
    const middle = screenToFlowPosition({ x: at.clientX, y: at.clientY });
    const made = tableAddedAt(schema, t("schemaNewTable"), middle);
    const name = made.picked?.kind === "table" ? made.picked.name : "";
    const linked = related(made.schema, { table: state.fromNode.id, index }, name);
    change(linked ?? made);
    setNaming({ kind: "table", name });
  };

  return (
    <div className="flex min-h-0 min-w-0 flex-1">
      <Palette<"table">
        groups={[{ title: t("schemaTables"), items: [{ key: "table", label: t("schemaTable") }] }]}
        sketch={() => <Sketch paths={[TABLE_SKETCH]} />}
        onAdd={() => added(middleOfPane())}
        onDrop={landed}
      />
      {/* biome-ignore lint/a11y/noStaticElementInteractions: the keys belong to the canvas inside */}
      <div className="flex min-h-0 min-w-0 flex-1 flex-col" onKeyDown={keyed}>
        <SchemaToolbar editing={editing} laid={laid} removal={removal} />
        <div ref={pane} className="relative min-h-0 flex-1 bg-desk">
          <ReactFlow<TableNode, WireEdge>
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
            onEdgeClick={(_, edge) => pick({ kind: "relation", index: Number(edge.id) })}
            onEdgeDoubleClick={(_, edge) => setNaming({ kind: "relation", index: Number(edge.id) })}
            onPaneClick={() => pick(null)}
            onConnect={(made) => joined(made.source, made.sourceHandle, made.target)}
            onConnectEnd={ended}
          >
            <Background variant={BackgroundVariant.Dots} gap={16} />
          </ReactFlow>
        </div>
      </div>
    </div>
  );
}
