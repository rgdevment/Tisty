import {
  BaseEdge,
  type Edge,
  EdgeLabelRenderer,
  type EdgeProps,
  type InternalNode,
  useInternalNode,
} from "@xyflow/react";
import { routeOf } from "./flowroutes";
import type { Rect } from "./graphlayout";
import Renaming from "./Renaming";

export type WireData = {
  readonly label: string;
  readonly title: string;
  readonly dashed: boolean;
  readonly thick: boolean;
  readonly naming: boolean;
  readonly field: string;
  readonly onPick: () => void;
  readonly onName: () => void;
  readonly onNamed: (text: string | null) => void;
  // A class relation's cardinalities, written by the ends they belong to.
  readonly startLabel?: string;
  readonly endLabel?: string;
};

export type WireEdge = Edge<WireData, "wire">;

const rectOf = (node: InternalNode): Rect => ({
  x: node.internals.positionAbsolute.x,
  y: node.internals.positionAbsolute.y,
  wide: node.measured.width ?? node.width ?? 0,
  tall: node.measured.height ?? node.height ?? 0,
});

/** An arrow that follows its shapes as they move, between the sides that face each other. */
export function Wire({
  id,
  source,
  target,
  data,
  selected,
  markerStart,
  markerEnd,
}: EdgeProps<WireEdge>) {
  const from = useInternalNode(source);
  const to = useInternalNode(target);
  if (!from || !to || !data) return null;
  const route = routeOf(rectOf(from), rectOf(to));
  const ends = [
    { said: data.startLabel, at: route.from },
    { said: data.endLabel, at: route.to },
  ];
  return (
    <>
      <BaseEdge
        id={id}
        path={route.path}
        markerStart={markerStart}
        markerEnd={markerEnd}
        interactionWidth={14}
        style={{
          stroke: "var(--color-accent)",
          strokeWidth: (data.thick ? 2.6 : 1.6) + (selected ? 1 : 0),
          strokeDasharray: data.dashed ? "6 4" : undefined,
        }}
      />
      <EdgeLabelRenderer>
        <div
          className="nodrag nopan absolute"
          style={{
            transform: `translate(-50%, -50%) translate(${route.labelX}px, ${route.labelY}px)`,
            pointerEvents: "all",
          }}
        >
          {data.naming ? (
            <Renaming text={data.label} label={data.field} onDone={data.onNamed} className="w-32" />
          ) : (
            <button
              type="button"
              aria-label={data.label ? undefined : data.title}
              onClick={data.onPick}
              onDoubleClick={data.onName}
              className={
                data.label
                  ? `rounded-full border bg-desk px-2 text-[11px] text-accent ${selected ? "border-accent" : "border-transparent"}`
                  : "block h-3 w-3 rounded-full opacity-0 focus:opacity-100 focus:ring-2 focus:ring-accent"
              }
            >
              {data.label}
            </button>
          )}
        </div>
        {ends.map(({ said, at }) =>
          said ? (
            <span
              key={`${at.x} ${at.y}`}
              className="absolute text-[11px] text-soft"
              style={{ transform: `translate(${at.x + 6}px, ${at.y - 18}px)` }}
            >
              {said}
            </span>
          ) : null,
        )}
      </EdgeLabelRenderer>
    </>
  );
}
