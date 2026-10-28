import { type PointerEvent as Pressed, type ReactNode, useRef, useState } from "react";
import { useSay } from "./handing";

type Point = { readonly x: number; readonly y: number };

export interface PaletteGroup<Key extends string> {
  readonly title: string;
  readonly items: readonly { readonly key: Key; readonly label: string }[];
}

// Less than this is a press, not a drag.
const SLACK = 4;

/** A small drawing of what a piece puts down, as wide as a palette cell. */
export function Sketch({ paths }: Readonly<{ paths: readonly string[] }>) {
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
      {paths.map((d) => (
        <path key={d} d={d} />
      ))}
    </svg>
  );
}

/** The pieces to drag onto the canvas, or to press and find in the middle of it. */
export default function Palette<Key extends string>({
  groups,
  sketch,
  hint,
  onAdd,
  onDrop,
}: Readonly<{
  groups: readonly PaletteGroup<Key>[];
  sketch: (key: Key) => ReactNode;
  hint: string;
  onAdd: (key: Key) => void;
  onDrop: (key: Key, at: Point) => void;
}>) {
  const t = useSay();
  const [ghost, setGhost] = useState<{ key: Key; at: Point } | null>(null);
  // A drag let go over its own button still ends in a click, which must not add a second piece.
  const swallowed = useRef(false);

  // The drag follows the pointer: on Windows the app holds the window's own drag and drop for files.
  const carry = (key: Key, e: Pressed) => {
    const from = { x: e.clientX, y: e.clientY };
    let moved = false;
    swallowed.current = false;
    const move = (m: PointerEvent) => {
      moved ||= Math.hypot(m.clientX - from.x, m.clientY - from.y) > SLACK;
      if (moved) setGhost({ key, at: { x: m.clientX, y: m.clientY } });
    };
    const up = (u: PointerEvent) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      setGhost(null);
      if (!moved) return;
      swallowed.current = true;
      onDrop(key, { x: u.clientX, y: u.clientY });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <aside
      aria-label={t("flowPalette")}
      className="scroller flex w-[212px] shrink-0 flex-col gap-3 overflow-y-auto border-r border-hair bg-rail p-3"
    >
      {groups.map((group) => (
        <section key={group.title} aria-label={group.title} className="flex flex-col gap-1">
          <h3 className="m-0 text-[11.5px] font-semibold text-faint">{group.title}</h3>
          <div className="grid grid-cols-3 gap-1">
            {group.items.map((item) => (
              <button
                key={item.key}
                type="button"
                onPointerDown={(e) => carry(item.key, e)}
                onClick={() => {
                  if (swallowed.current) swallowed.current = false;
                  else onAdd(item.key);
                }}
                className="flex h-[58px] cursor-grab touch-none flex-col items-center justify-center gap-1 rounded-md text-[10.5px] leading-tight text-soft select-none hover:bg-hover hover:text-ink"
              >
                {sketch(item.key)}
                {item.label}
              </button>
            ))}
          </div>
        </section>
      ))}
      <p className="m-0 mt-auto text-[11.5px] leading-snug text-faint">{hint}</p>
      {ghost && (
        <div
          aria-hidden="true"
          className="pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-1/2 rounded-md bg-panel p-1 text-accent shadow"
          style={{ left: ghost.at.x, top: ghost.at.y }}
        >
          {sketch(ghost.key)}
        </div>
      )}
    </aside>
  );
}
