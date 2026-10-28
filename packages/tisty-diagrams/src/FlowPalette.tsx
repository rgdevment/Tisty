import { type PointerEvent as Pressed, useRef, useState } from "react";
import { outlineOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import { useSay } from "./handing";
import { PALETTE, shapeName } from "./paletteshapes";

type Point = { readonly x: number; readonly y: number };

// Less than this is a press, not a drag.
const SLACK = 4;

function Preview({ shape }: Readonly<{ shape: Shape }>) {
  const outline = outlineOf(shape, 34, 22);
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
}: Readonly<{ onAdd: (shape: Shape) => void; onDrop: (shape: Shape, at: Point) => void }>) {
  const t = useSay();
  const [ghost, setGhost] = useState<{ shape: Shape; at: Point } | null>(null);
  // A drag let go over its own button still ends in a click, which must not add a second shape.
  const swallowed = useRef(false);

  // The drag follows the pointer: on Windows the app holds the window's own drag and drop for files.
  const carry = (shape: Shape, e: Pressed) => {
    const from = { x: e.clientX, y: e.clientY };
    let moved = false;
    swallowed.current = false;
    const move = (m: PointerEvent) => {
      moved ||= Math.hypot(m.clientX - from.x, m.clientY - from.y) > SLACK;
      if (moved) setGhost({ shape, at: { x: m.clientX, y: m.clientY } });
    };
    const up = (u: PointerEvent) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      setGhost(null);
      if (!moved) return;
      swallowed.current = true;
      onDrop(shape, { x: u.clientX, y: u.clientY });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <aside
      aria-label={t("flowPalette")}
      className="scroller flex w-[212px] shrink-0 flex-col gap-3 overflow-y-auto border-r border-hair bg-rail p-3"
    >
      {PALETTE.map((group) => (
        <section key={group.title} aria-label={t(group.title)} className="flex flex-col gap-1">
          <h3 className="m-0 text-[11.5px] font-semibold text-faint">{t(group.title)}</h3>
          <div className="grid grid-cols-3 gap-1">
            {group.shapes.map((shape) => (
              <button
                key={shape}
                type="button"
                onPointerDown={(e) => carry(shape, e)}
                onClick={() => {
                  if (swallowed.current) swallowed.current = false;
                  else onAdd(shape);
                }}
                className="flex h-[58px] cursor-grab touch-none flex-col items-center justify-center gap-1 rounded-md text-[10.5px] leading-tight text-soft select-none hover:bg-hover hover:text-ink"
              >
                <Preview shape={shape} />
                {shapeName(shape, t)}
              </button>
            ))}
          </div>
        </section>
      ))}
      <p className="m-0 mt-auto text-[11.5px] leading-snug text-faint">{t("flowPaletteHint")}</p>
      {ghost && (
        <div
          aria-hidden="true"
          className="pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-1/2 rounded-md bg-panel p-1 text-accent shadow"
          style={{ left: ghost.at.x, top: ghost.at.y }}
        >
          <Preview shape={ghost.shape} />
        </div>
      )}
    </aside>
  );
}
