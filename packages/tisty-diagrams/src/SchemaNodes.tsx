import { Handle, type Node, type NodeProps, Position } from "@xyflow/react";
import Renaming from "./Renaming";
import { SIDES } from "./ShapeNodes";
import type { Table } from "./schema";
import { HEAD, ROW } from "./schemalayout";

export type TableData = {
  readonly table: Table;
  readonly wide: number;
  readonly tall: number;
  // The column chosen in this table, if one is.
  readonly column: number | null;
  readonly naming: boolean;
  readonly label: string;
  readonly linkTitle: string;
  readonly onPick: () => void;
  readonly onPickColumn: (index: number) => void;
  readonly onName: () => void;
  readonly onNamed: (text: string | null) => void;
};

export type TableNode = Node<TableData, "table">;

/** The id a column's dot carries, which says which column a relation is drawn from. */
export const columnHandle = (index: number) => `column-${index}`;

/** The column a dot stands for, or null for a dot on a side of the card. */
export const columnOfHandle = (handle: string | null | undefined): number | null => {
  const found = /^column-(\d+)$/.exec(handle ?? "");
  return found ? Number(found[1]) : null;
};

/** The handles React Flow measures a card by: a dot for each column, and the sides relations meet. */
export const tableHandles = (wide: number, tall: number, columns: number) => [
  ...SIDES.map((side) => ({
    id: side,
    type: "source" as const,
    position: side,
    x: side === Position.Right ? wide : side === Position.Left ? 0 : wide / 2,
    y: side === Position.Bottom ? tall : side === Position.Top ? 0 : tall / 2,
    width: 1,
    height: 1,
  })),
  ...Array.from({ length: columns }, (_, index) => ({
    id: columnHandle(index),
    type: "source" as const,
    position: Position.Right,
    x: wide - 12,
    y: HEAD + index * ROW + ROW / 2 - 4,
    width: 10,
    height: 10,
  })),
];

/** A table as a card: its name to drag it by, its columns to choose, and a dot on each to relate it. */
export function TableCard({ id, data, selected }: NodeProps<TableNode>) {
  const { table } = data;
  return (
    <div
      data-node={id}
      data-table={id}
      className={`relative overflow-hidden rounded-lg border bg-panel text-[12.5px] text-ink shadow-sm ${selected ? "border-accent" : "border-line"}`}
      style={{ width: data.wide, height: data.tall }}
    >
      <div
        className="flex items-center border-b border-hair bg-hover px-3"
        style={{ height: HEAD }}
      >
        {data.naming ? (
          <Renaming text={table.name} label={data.label} onDone={data.onNamed} className="w-full" />
        ) : (
          <button
            type="button"
            onClick={(e) => {
              if (!e.ctrlKey && !e.metaKey) data.onPick();
            }}
            onDoubleClick={data.onName}
            className="w-full cursor-[inherit] truncate bg-transparent text-left font-semibold"
          >
            {table.name}
          </button>
        )}
      </div>
      {table.columns.map((column, index) => (
        <div
          // biome-ignore lint/suspicious/noArrayIndexKey: a column is its place in the table; two may share a name while typed.
          key={index}
          className="relative flex items-center"
          style={{ height: ROW }}
        >
          <button
            type="button"
            aria-pressed={data.column === index}
            className={`nodrag flex h-full flex-1 items-center gap-2 pr-7 pl-3 text-left ${data.column === index ? "bg-accent-soft" : ""}`}
            // A column is chosen alone, not the card React Flow would choose under it.
            onClick={(e) => {
              e.stopPropagation();
              data.onPickColumn(index);
            }}
          >
            <span className="w-6 text-[10px] font-bold text-accent">{column.keys.join(",")}</span>
            <span className="flex-1 truncate">
              {column.name}
              {column.required ? " *" : ""}
            </span>
            <span className="font-mono text-[11.5px] text-faint">{column.type}</span>
          </button>
          <Handle
            id={columnHandle(index)}
            type="source"
            position={Position.Right}
            title={data.linkTitle}
            className="right-1.5! h-2.5! w-2.5! cursor-crosshair! border-accent! bg-desk!"
          />
        </div>
      ))}
      {SIDES.map((side) => (
        <Handle
          key={side}
          id={side}
          type="source"
          position={side}
          isConnectable={false}
          className="pointer-events-none opacity-0"
        />
      ))}
    </div>
  );
}
