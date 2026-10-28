import { type KeyboardEvent, type PointerEvent, type ReactNode, useRef, useState } from "react";
import Board from "./Board";
import type { Size } from "./framing";
import { edgeOf, type Rect } from "./graphlayout";

export interface Node {
  readonly id: string;
  readonly rect: Rect;
  readonly picked: boolean;
  readonly look: string;
  readonly style?: React.CSSProperties;
  readonly children: ReactNode;
}

export interface Wire {
  readonly key: string;
  readonly from: string;
  readonly to: string;
  readonly label: string;
  readonly dashed: boolean;
  readonly thick: boolean;
  readonly arrow: boolean;
  readonly picked: boolean;
}

export interface Frame {
  readonly key: string;
  readonly rect: Rect;
  readonly title: string;
  readonly picked: boolean;
}

const nodeUnder = (x: number, y: number) =>
  document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-node]")?.dataset.node ?? null;

// The board is scaled; a point under the pointer is read in the drawing's own units.
const pointIn = (e: PointerEvent, inside: Element) => {
  const room = inside.closest<HTMLElement>(".diagram-drawn");
  const box = room?.getBoundingClientRect();
  const scale = room?.offsetWidth ? (box?.width ?? 0) / room.offsetWidth : 1;
  return { x: (e.clientX - (box?.left ?? 0)) / scale, y: (e.clientY - (box?.top ?? 0)) / scale };
};

/** Boxes to choose and to join by dragging a dot onto another, wires to choose by their label, and frames around groups. */
export default function GraphBoard({
  size,
  nodes,
  wires,
  frames,
  notice,
  linkHint,
  onPickNode,
  onPickWire,
  onPickFrame,
  onLink,
  onDelete,
}: Readonly<{
  size: Size;
  nodes: readonly Node[];
  wires: readonly Wire[];
  frames: readonly Frame[];
  notice: ReactNode;
  linkHint: string;
  onPickNode: (id: string) => void;
  onPickWire: (key: string) => void;
  onPickFrame: (key: string) => void;
  onLink: (from: string, to: string) => void;
  onDelete: () => void;
}>) {
  const rects = new Map(nodes.map((one) => [one.id, one.rect]));
  const [linking, setLinking] = useState<{ from: string; at: { x: number; y: number } } | null>(
    null,
  );
  const held = useRef<string | null>(null);

  const keyed = (e: KeyboardEvent) => {
    if (e.key !== "Delete" && e.key !== "Backspace") return;
    if (!(e.target as Element).closest("[data-picks]")) return;
    e.preventDefault();
    onDelete();
  };

  const ends = (from: Rect, to: Rect) => ({ a: edgeOf(from, to), b: edgeOf(to, from) });
  const letGo = (e: PointerEvent) => {
    const from = held.current;
    held.current = null;
    setLinking(null);
    const to = nodeUnder(e.clientX, e.clientY);
    if (from && to && to !== from) onLink(from, to);
  };

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: Delete belongs to what is chosen, whose button holds the focus.
    <div className="relative flex min-w-0 flex-1" onKeyDown={keyed}>
      <Board size={size} notice={notice}>
        <svg
          aria-hidden="true"
          className="absolute top-0 left-0 overflow-visible"
          width={size.wide}
          height={size.tall}
        >
          <defs>
            <marker
              id="graph-arrow"
              viewBox="0 0 10 10"
              refX="9"
              refY="5"
              markerWidth="7"
              markerHeight="7"
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--tisty-accent)" />
            </marker>
          </defs>
          {frames.map((one) => (
            <rect
              key={one.key}
              x={one.rect.x}
              y={one.rect.y}
              width={one.rect.wide}
              height={one.rect.tall}
              rx={10}
              fill="var(--tisty-hover)"
              stroke={one.picked ? "var(--tisty-accent)" : "var(--tisty-line)"}
              strokeDasharray="5 4"
            />
          ))}
          {wires.map((one) => {
            const from = rects.get(one.from);
            const to = rects.get(one.to);
            if (!from || !to) return null;
            const { a, b } = ends(from, to);
            return (
              <line
                key={one.key}
                x1={a.x}
                y1={a.y}
                x2={b.x}
                y2={b.y}
                stroke="var(--tisty-accent)"
                strokeWidth={(one.thick ? 2.6 : 1.6) + (one.picked ? 1 : 0)}
                strokeDasharray={one.dashed ? "6 4" : undefined}
                markerEnd={one.arrow ? "url(#graph-arrow)" : undefined}
              />
            );
          })}
          {linking && rects.get(linking.from) && (
            <line
              x1={(rects.get(linking.from)?.x ?? 0) + (rects.get(linking.from)?.wide ?? 0) / 2}
              y1={(rects.get(linking.from)?.y ?? 0) + (rects.get(linking.from)?.tall ?? 0) / 2}
              x2={linking.at.x}
              y2={linking.at.y}
              stroke="var(--tisty-accent)"
              strokeWidth={2}
              strokeDasharray="4 4"
            />
          )}
        </svg>
        {frames.map((one) => (
          <button
            key={one.key}
            type="button"
            data-picks
            aria-pressed={one.picked}
            className="absolute truncate px-2 text-left text-[11.5px] font-semibold text-soft"
            style={{ left: one.rect.x + 6, top: one.rect.y + 4, maxWidth: one.rect.wide - 12 }}
            onClick={() => onPickFrame(one.key)}
          >
            {one.title}
          </button>
        ))}
        {wires.map((one) => {
          const from = rects.get(one.from);
          const to = rects.get(one.to);
          if (!from || !to) return null;
          const { a, b } = ends(from, to);
          return (
            <button
              key={one.key}
              type="button"
              data-picks
              aria-pressed={one.picked}
              aria-label={one.label || `${one.from} → ${one.to}`}
              className={`absolute -translate-x-1/2 -translate-y-1/2 rounded-full border bg-desk px-1.5 text-[11px] whitespace-nowrap text-accent ${
                one.label || one.picked ? "border-accent" : "border-transparent"
              }`}
              style={{ left: (a.x + b.x) / 2, top: (a.y + b.y) / 2, minWidth: 14, minHeight: 14 }}
              onClick={() => onPickWire(one.key)}
            >
              {one.label}
            </button>
          );
        })}
        {nodes.map((one) => (
          <div
            key={one.id}
            data-node={one.id}
            className="absolute"
            style={{
              left: one.rect.x,
              top: one.rect.y,
              width: one.rect.wide,
              height: one.rect.tall,
            }}
          >
            <button
              type="button"
              data-picks
              aria-pressed={one.picked}
              className={`box-border h-full w-full ${one.look} ${one.picked ? "ring-2 ring-accent ring-offset-2 ring-offset-desk" : ""}`}
              style={one.style}
              onClick={() => onPickNode(one.id)}
            >
              {one.children}
            </button>
            <span
              data-grip
              role="presentation"
              title={linkHint}
              className="absolute -right-1.5 top-1/2 h-3 w-3 -translate-y-1/2 cursor-crosshair rounded-full border border-accent bg-desk"
              onPointerDown={(e) => {
                held.current = one.id;
                setLinking({ from: one.id, at: pointIn(e, e.currentTarget) });
                e.currentTarget.setPointerCapture?.(e.pointerId);
              }}
              onPointerMove={(e) => {
                if (held.current && e.buttons !== 0)
                  setLinking({ from: held.current, at: pointIn(e, e.currentTarget) });
              }}
              onPointerUp={letGo}
              onPointerCancel={() => {
                held.current = null;
                setLinking(null);
              }}
            />
          </div>
        ))}
      </Board>
    </div>
  );
}
