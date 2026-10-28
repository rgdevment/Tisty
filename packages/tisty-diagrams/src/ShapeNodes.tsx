import { Handle, type Node, type NodeProps, Position } from "@xyflow/react";
import { outlineOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import Renaming from "./Renaming";

export const SIDES = [Position.Top, Position.Right, Position.Bottom, Position.Left] as const;

export type ShapeData = {
  readonly text: string;
  readonly shape: Shape;
  readonly wide: number;
  readonly tall: number;
  readonly naming: boolean;
  readonly label: string;
  readonly onPick: () => void;
  readonly onName: () => void;
  readonly onNamed: (text: string | null) => void;
};

export type FrameData = {
  readonly title: string;
  readonly picked: boolean;
  readonly onPick: () => void;
};

export type ShapeNode = Node<ShapeData, "shape">;
export type FrameNode = Node<FrameData, "frame">;

/** A shape drawn from its outline, its words over it, a dot on each side to start an arrow from. */
export function ShapeBox({ id, data, selected }: NodeProps<ShapeNode>) {
  const outline = outlineOf(data.shape, data.wide, data.tall);
  const tone = selected ? "stroke-accent" : "stroke-line";
  return (
    <div data-node={id} className="group relative" style={{ width: data.wide, height: data.tall }}>
      <svg
        aria-hidden="true"
        width={data.wide}
        height={data.tall}
        className="absolute inset-0 overflow-visible"
      >
        {outline.body && (
          <path d={outline.body} className={`fill-panel ${tone}`} strokeWidth={selected ? 2 : 1} />
        )}
        {outline.lines.map((d) => (
          <path key={d} d={d} fill="none" className={tone} strokeWidth={selected ? 2 : 1} />
        ))}
      </svg>
      {data.naming ? (
        <div className="absolute inset-0 flex items-center justify-center px-2">
          <Renaming text={data.text} label={data.label} onDone={data.onNamed} className="w-full" />
        </div>
      ) : (
        <button
          type="button"
          onClick={data.onPick}
          onDoubleClick={data.onName}
          className="absolute inset-0 flex cursor-[inherit] items-center justify-center bg-transparent px-3 text-[13px] text-ink"
        >
          <span className="truncate" style={{ transform: `translateY(${outline.drop}px)` }}>
            {data.text}
          </span>
        </button>
      )}
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

/** The frame of a group, behind its shapes, chosen by its title. */
export function FrameBox({ id, data }: NodeProps<FrameNode>) {
  return (
    <div
      data-frame={id}
      className={`h-full w-full rounded-[10px] border border-dashed bg-hover ${data.picked ? "border-accent" : "border-line"}`}
    >
      <button
        type="button"
        onClick={data.onPick}
        className="m-1.5 rounded px-1.5 text-[11.5px] font-semibold text-soft hover:bg-hover"
      >
        {data.title}
      </button>
    </div>
  );
}
