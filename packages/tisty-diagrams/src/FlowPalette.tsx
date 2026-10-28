import { type PointerEvent as Pressed, useRef, useState } from "react";
import { outlineOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import { useSay } from "./handing";
import { PALETTE, shapeName } from "./paletteshapes";
import type { Word } from "./words";

type Point = { readonly x: number; readonly y: number };

/** What the palette puts on the canvas: a shape, or a lane running down or across. */
export type Piece = Shape | "column" | "row";

const LANES: readonly { readonly piece: "column" | "row"; readonly word: Word }[] = [
  { piece: "column", word: "flowLaneColumn" },
  { piece: "row", word: "flowLaneRow" },
];

// Less than this is a press, not a drag.
const SLACK = 4;

function Preview({ piece }: Readonly<{ piece: Piece }>) {
  if (piece === "column" || piece === "row")
    return (
      <svg
        aria-hidden="true"
        width="36"
        height="24"
        viewBox="-1 -1 36 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.3"
      >
        <path
          d={
            piece === "column"
              ? "M2,1 H32 V21 H2 Z M2,6 H32 M12,1 V21 M22,1 V21"
              : "M2,1 H32 V21 H2 Z M7,1 V21 M2,8 H32 M2,15 H32"
          }
        />
      </svg>
    );
  const outline = outlineOf(piece, 34, 22);
  return (
    <svg
      aria-hidden="true"
      width="36"
      height="24"
      viewBox="-1 -1 36 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      {outline.body && <path d={outline.body} />}
      {outline.lines.map((d) => (
        <path key={d} d={d} />
      ))}
      {!outline.body && !outline.lines.length && <path d="M8,7 H26 M8,12 H26 M8,17 H19" />}
    </svg>
  );
}

/** The shapes to drag onto the canvas, or to press and find in the middle of it. */
export default function FlowPalette({
  onAdd,
  onDrop,
}: Readonly<{ onAdd: (piece: Piece) => void; onDrop: (piece: Piece, at: Point) => void }>) {
  const t = useSay();
  const [ghost, setGhost] = useState<{ piece: Piece; at: Point } | null>(null);
  // A drag let go over its own button still ends in a click, which must not add a second shape.
  const swallowed = useRef(false);

  // The drag follows the pointer: on Windows the app holds the window's own drag and drop for files.
  const carry = (piece: Piece, e: Pressed) => {
    const from = { x: e.clientX, y: e.clientY };
    let moved = false;
    swallowed.current = false;
    const move = (m: PointerEvent) => {
      moved ||= Math.hypot(m.clientX - from.x, m.clientY - from.y) > SLACK;
      if (moved) setGhost({ piece, at: { x: m.clientX, y: m.clientY } });
    };
    const up = (u: PointerEvent) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      setGhost(null);
      if (!moved) return;
      swallowed.current = true;
      onDrop(piece, { x: u.clientX, y: u.clientY });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  const pieceButton = (piece: Piece, label: string) => (
    <button
      key={piece}
      type="button"
      onPointerDown={(e) => carry(piece, e)}
      onClick={() => {
        if (swallowed.current) swallowed.current = false;
        else onAdd(piece);
      }}
      className="flex h-[58px] cursor-grab touch-none flex-col items-center justify-center gap-1 rounded-md text-[10.5px] leading-tight text-soft select-none hover:bg-hover hover:text-ink"
    >
      <Preview piece={piece} />
      {label}
    </button>
  );

  return (
    <aside
      aria-label={t("flowPalette")}
      className="scroller flex w-[212px] shrink-0 flex-col gap-3 overflow-y-auto border-r border-hair bg-rail p-3"
    >
      {PALETTE.map((group) => (
        <section key={group.title} aria-label={t(group.title)} className="flex flex-col gap-1">
          <h3 className="m-0 text-[11.5px] font-semibold text-faint">{t(group.title)}</h3>
          <div className="grid grid-cols-3 gap-1">
            {group.shapes.map((shape) => pieceButton(shape, shapeName(shape, t)))}
          </div>
        </section>
      ))}
      <section aria-label={t("flowLanes")} className="flex flex-col gap-1">
        <h3 className="m-0 text-[11.5px] font-semibold text-faint">{t("flowLanes")}</h3>
        <div className="grid grid-cols-3 gap-1">
          {LANES.map((one) => pieceButton(one.piece, t(one.word)))}
        </div>
      </section>
      <p className="m-0 mt-auto text-[11.5px] leading-snug text-faint">{t("flowPaletteHint")}</p>
      {ghost && (
        <div
          aria-hidden="true"
          className="pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-1/2 rounded-md bg-panel p-1 text-accent shadow"
          style={{ left: ghost.at.x, top: ghost.at.y }}
        >
          <Preview piece={ghost.piece} />
        </div>
      )}
    </aside>
  );
}
