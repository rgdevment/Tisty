import { Handle, type Node, type NodeProps, NodeResizer, Position } from "@xyflow/react";
import { outlineOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import type { Look } from "./flowstyles";
import type { Rect } from "./graphlayout";
import Renaming from "./Renaming";

export const SIDES = [Position.Top, Position.Right, Position.Bottom, Position.Left] as const;

export type ShapeData = {
  readonly text: string;
  readonly shape: Shape;
  readonly wide: number;
  readonly tall: number;
  readonly look: Look | undefined;
  readonly naming: boolean;
  readonly label: string;
  readonly onPick: () => void;
  readonly onName: () => void;
  readonly onNamed: (text: string | null) => void;
};

export type LaneData = {
  readonly title: string;
  readonly across: boolean;
  readonly picked: boolean;
  readonly onPick: () => void;
  readonly onResizing: () => void;
  readonly onResized: (box: Rect) => void;
};

export type FrameData = {
  readonly title: string;
  readonly picked: boolean;
  readonly onPick: () => void;
};

export type ShapeNode = Node<ShapeData, "shape">;
export type FrameNode = Node<FrameData, "frame">;
export type LaneNode = Node<LaneData, "lane">;

/** A shape drawn from its outline, its words over it, a dot on each side to start an arrow from. */
export function ShapeBox({ id, data, selected }: NodeProps<ShapeNode>) {
  const outline = outlineOf(data.shape, data.wide, data.tall);
  const tone = selected ? "stroke-accent" : "stroke-line";
  // A colour from the node's `style` line wins over the theme's, except the accent of being chosen.
  const painted = {
    fill: data.look?.fill,
    stroke: selected ? undefined : data.look?.stroke,
  };
  return (
    <div data-node={id} className="group relative" style={{ width: data.wide, height: data.tall }}>
      <svg
        aria-hidden="true"
        width={data.wide}
        height={data.tall}
        className="absolute inset-0 overflow-visible"
      >
        {outline.body && (
          <path
            d={outline.body}
            className={`fill-panel ${tone}`}
            style={painted}
            strokeWidth={selected ? 2 : 1}
          />
        )}
        {outline.lines.map((d) => (
          <path
            key={d}
            d={d}
            fill="none"
            className={tone}
            style={{ stroke: painted.stroke }}
            strokeWidth={selected ? 2 : 1}
          />
        ))}
      </svg>
      {data.naming ? (
        <div className="absolute inset-0 flex items-center justify-center px-2">
          <Renaming text={data.text} label={data.label} onDone={data.onNamed} className="w-full" />
        </div>
      ) : (
        <button
          type="button"
          onClick={(e) => {
            if (!e.ctrlKey && !e.metaKey) data.onPick();
          }}
          onDoubleClick={data.onName}
          className="absolute inset-0 flex cursor-[inherit] items-center justify-center bg-transparent px-3 text-[13px] text-ink"
          style={{ color: data.look?.color }}
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

/** A group drawn as a lane: a column or a row its shapes sit in, moved and sized by hand. */
export function LaneBox({ id, data }: NodeProps<LaneNode>) {
  const band = data.across
    ? "absolute inset-y-0 left-0 flex w-8 items-center justify-center border-r"
    : "flex h-8 items-center border-b px-2";
  return (
    <div
      data-lane={id}
      className={`relative h-full w-full rounded-md border bg-hover ${data.picked ? "border-accent" : "border-line"}`}
    >
      <NodeResizer
        isVisible={data.picked}
        minWidth={120}
        minHeight={80}
        lineClassName="border-accent!"
        handleClassName="h-2.5! w-2.5! border-sheet! bg-accent!"
        onResizeStart={data.onResizing}
        onResizeEnd={(_, at) =>
          data.onResized({ x: at.x, y: at.y, wide: at.width, tall: at.height })
        }
      />
      <div className={`${band} border-line`}>
        <button
          type="button"
          onClick={data.onPick}
          className={`rounded px-1.5 text-[12px] font-semibold text-soft hover:bg-hover ${data.across ? "[writing-mode:vertical-rl] rotate-180" : ""}`}
        >
          {data.title}
        </button>
      </div>
    </div>
  );
}
