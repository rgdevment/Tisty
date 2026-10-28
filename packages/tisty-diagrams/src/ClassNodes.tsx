import { Handle, type Node, type NodeProps } from "@xyflow/react";
import type { ClassBox } from "./classes";
import type { Look } from "./flowstyles";
import Renaming from "./Renaming";
import { SIDES } from "./ShapeNodes";
import { CLASS_HEAD, CLASS_LINE, type Head } from "./scenes";

export type ClassData = {
  readonly box: ClassBox;
  readonly wide: number;
  readonly tall: number;
  readonly look: Look | undefined;
  readonly naming: boolean;
  readonly label: string;
  readonly onPick: () => void;
  readonly onName: () => void;
  readonly onNamed: (text: string | null) => void;
};

export type ClassNode = Node<ClassData, "class">;

/** The id of the marker each head is drawn with on the canvas. */
export const headId = (head: Head) => `tisty-head-${head}`;

// A hollow head is filled with the panel so the line does not show through it.
const HEADS: readonly (readonly [Head, string, string])[] = [
  ["arrow", "M 0 0 L 10 5 L 0 10 z", "var(--color-accent)"],
  ["open", "M 0 0 L 10 5 L 0 10", "none"],
  ["triangle", "M 0 0 L 10 5 L 0 10 z", "var(--color-panel)"],
  ["diamond", "M 0 5 L 5 0 L 10 5 L 5 10 z", "var(--color-panel)"],
  ["solidDiamond", "M 0 5 L 5 0 L 10 5 L 5 10 z", "var(--color-accent)"],
];

/** The heads relations end in, once for the whole canvas. */
export function HeadMarkers() {
  return (
    <svg aria-hidden="true" width="0" height="0" className="absolute">
      <defs>
        {HEADS.map(([head, d, fill]) => (
          <marker
            key={head}
            id={headId(head)}
            viewBox="-1 -1 12 12"
            refX="10"
            refY="5"
            markerWidth="12"
            markerHeight="12"
            orient="auto-start-reverse"
          >
            <path d={d} style={{ fill, stroke: "var(--color-accent)" }} strokeWidth="1" />
          </marker>
        ))}
      </defs>
    </svg>
  );
}

/** A class as a card: its kind and name on top, its members below, a dot on each side to relate it. */
export function ClassCard({ id, data, selected }: NodeProps<ClassNode>) {
  const { box, look } = data;
  const top = CLASS_HEAD + (box.note ? 14 : 0);
  return (
    <div
      data-node={id}
      className={`group relative flex flex-col overflow-hidden rounded-md border bg-panel text-ink ${selected ? "border-accent" : "border-line"}`}
      style={{
        width: data.wide,
        height: data.tall,
        background: look?.fill,
        borderColor: selected ? undefined : look?.stroke,
        color: look?.color,
      }}
    >
      <div
        className="flex shrink-0 flex-col items-center justify-center border-b border-hair bg-hover px-2"
        style={{ height: top }}
      >
        {data.naming ? (
          <Renaming text={box.name} label={data.label} onDone={data.onNamed} className="w-full" />
        ) : (
          <button
            type="button"
            onClick={(e) => {
              if (!e.ctrlKey && !e.metaKey) data.onPick();
            }}
            onDoubleClick={data.onName}
            className="flex w-full cursor-[inherit] flex-col items-center bg-transparent text-[13px] font-semibold"
          >
            {box.note && (
              <small className="text-[10.5px] font-normal text-faint">«{box.note}»</small>
            )}
            <span className="max-w-full truncate">{box.name}</span>
          </button>
        )}
      </div>
      {box.members.map((member) => (
        <span
          key={member.line}
          className="truncate px-3 font-mono text-[11.5px] text-soft"
          style={{ lineHeight: `${CLASS_LINE}px`, color: look?.color }}
        >
          {member.text}
        </span>
      ))}
      {SIDES.map((side) => (
        <Handle
          key={side}
          id={side}
          type="source"
          position={side}
          className="h-2.5! w-2.5! border-sheet! bg-accent! opacity-0 transition-opacity group-hover:opacity-100"
        />
      ))}
    </div>
  );
}
