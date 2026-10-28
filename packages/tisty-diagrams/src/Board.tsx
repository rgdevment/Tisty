import { type ReactNode, useEffect, useRef, useState } from "react";
import { fitted, type Size, STEP, showing, type View, zoomedAt } from "./framing";
import { useHands, useSay } from "./handing";
import type { Zooming } from "./tools";

// What the board pans for is the empty room; a press on an idea or a control is theirs.
const OWN = "button, input, [data-idea], [data-grip]";

/** A room that shows something of a given size, moved by dragging or the wheel and zoomed under the pointer. */
export default function Board({
  size,
  notice,
  seen,
  bar,
  children,
}: Readonly<{
  size: Size | null;
  notice?: ReactNode;
  // A part to bring into sight when it changes, such as a table just added.
  seen?: { x: number; y: number; wide: number; tall: number } | null;
  // A bar of tools above the room, given how to zoom it, in place of the room's own zoom.
  bar?: (zooming: Zooming) => ReactNode;
  children: ReactNode;
}>) {
  const t = useSay();
  const { button } = useHands().looks;
  const room = useRef<HTMLElement>(null);
  const [view, setView] = useState<View | null>(null);
  const dragged = useRef<{ x: number; y: number } | null>(null);

  const roomSize = (): Size => ({
    wide: room.current?.clientWidth ?? 0,
    tall: room.current?.clientHeight ?? 0,
  });

  const fitAll = () => {
    if (size) setView(fitted(size, roomSize()));
  };

  const first = size !== null && view === null;
  useEffect(() => {
    if (first) fitAll();
  });

  const sought = seen ? `${seen.x} ${seen.y} ${seen.wide} ${seen.tall}` : "";
  // biome-ignore lint/correctness/useExhaustiveDependencies: the part is looked for when it changes, not on every drawing.
  useEffect(() => {
    if (seen) setView((was) => (was ? showing(was, seen, roomSize()) : was));
  }, [sought]);

  const zoom = (by: number) => {
    const middle = { x: roomSize().wide / 2, y: roomSize().tall / 2 };
    setView((was) => (was ? zoomedAt(was, by, middle) : was));
  };

  useEffect(() => {
    const here = room.current;
    if (!here) return;
    const wheel = (e: WheelEvent) => {
      e.preventDefault();
      const box = here.getBoundingClientRect();
      const point = { x: e.clientX - box.left, y: e.clientY - box.top };
      const by = e.deltaY < 0 ? STEP : 1 / STEP;
      setView((was) => {
        if (!was) return was;
        if (e.ctrlKey || e.metaKey) return zoomedAt(was, by, point);
        return { ...was, x: was.x - e.deltaX, y: was.y - e.deltaY };
      });
    };
    here.addEventListener("wheel", wheel, { passive: false });
    return () => here.removeEventListener("wheel", wheel);
  }, []);

  const letGo = () => {
    dragged.current = null;
  };

  const shown = (
    <section
      ref={room}
      aria-label={t("diagramCanvas")}
      className="relative min-h-0 min-w-0 flex-1 cursor-grab overflow-hidden bg-desk active:cursor-grabbing"
      onPointerDown={(e) => {
        if ((e.target as Element).closest(OWN)) return;
        dragged.current = { x: e.clientX, y: e.clientY };
        e.currentTarget.setPointerCapture?.(e.pointerId);
      }}
      onPointerMove={(e) => {
        const from = dragged.current;
        if (!from) return;
        // A release the board never heard, outside the window or mid alt-tab, still ends the drag.
        if (e.buttons === 0) return letGo();
        dragged.current = { x: e.clientX, y: e.clientY };
        setView((was) =>
          was ? { ...was, x: was.x + e.clientX - from.x, y: was.y + e.clientY - from.y } : was,
        );
      }}
      onPointerUp={letGo}
      onPointerCancel={letGo}
      onLostPointerCapture={letGo}
    >
      {size && view && (
        <div
          className="diagram-drawn absolute top-0 left-0 origin-top-left"
          style={{
            width: size.wide,
            height: size.tall,
            transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})`,
          }}
        >
          {children}
        </div>
      )}
      {notice}
      {!bar && (
        <div className="absolute right-3 bottom-3 flex items-center gap-1.5 rounded-lg bg-panel p-1 shadow">
          <button
            type="button"
            className={button}
            aria-label={t("diagramZoomOut")}
            onClick={() => zoom(1 / STEP)}
          >
            −
          </button>
          <span className="w-12 text-center text-[12px] text-soft tabular-nums">
            {Math.round((view?.scale ?? 1) * 100)}%
          </span>
          <button
            type="button"
            className={button}
            aria-label={t("diagramZoomIn")}
            onClick={() => zoom(STEP)}
          >
            +
          </button>
          <button type="button" className={button} onClick={fitAll}>
            {t("diagramFit")}
          </button>
        </div>
      )}
    </section>
  );
  if (!bar) return shown;
  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col">
      {bar({
        scale: view?.scale ?? 1,
        onOut: () => zoom(1 / STEP),
        onIn: () => zoom(STEP),
        onFit: fitAll,
      })}
      {shown}
    </div>
  );
}
