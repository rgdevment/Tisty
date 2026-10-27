import { useEffect, useRef, useState } from "react";
import { t } from "../../locales";
import { drawing } from "../diagrams";
import { button } from "../viewer/Glyphs";
import { fitted, type Size, STEP, sizeOf, type View, zoomedAt } from "./framing";

// Drawing on every key would lay out a half-typed diagram many times a second.
const SETTLE_MS = 250;

interface Drawn {
  svg: string;
  size: Size;
}

export default function Canvas({ source, dark }: { source: string; dark: boolean }) {
  const room = useRef<HTMLDivElement>(null);
  const [drawn, setDrawn] = useState<Drawn | null>(null);
  const [broken, setBroken] = useState(false);
  const [view, setView] = useState<View | null>(null);
  const dragged = useRef<{ x: number; y: number } | null>(null);

  useEffect(() => {
    let live = true;
    const timer = setTimeout(() => {
      void drawing(source, dark, () => live).then((svg) => {
        if (!live) return;
        const size = svg === null ? null : sizeOf(svg);
        setBroken(size === null);
        // The last good drawing stays while the text is half written.
        if (svg !== null && size) setDrawn({ svg, size });
      });
    }, SETTLE_MS);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [source, dark]);

  const roomSize = (): Size => ({
    wide: room.current?.clientWidth ?? 0,
    tall: room.current?.clientHeight ?? 0,
  });

  const fitAll = () => {
    if (drawn) setView(fitted(drawn.size, roomSize()));
  };

  const first = drawn !== null && view === null;
  useEffect(() => {
    if (first) fitAll();
  });

  const zoom = (by: number, point = { x: roomSize().wide / 2, y: roomSize().tall / 2 }) =>
    setView((was) => (was ? zoomedAt(was, by, point) : was));

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

  return (
    <section
      ref={room}
      aria-label={t("diagramCanvas")}
      className="relative min-w-0 flex-1 cursor-grab overflow-hidden bg-desk active:cursor-grabbing"
      onPointerDown={(e) => {
        if ((e.target as Element).closest("button")) return;
        dragged.current = { x: e.clientX, y: e.clientY };
        e.currentTarget.setPointerCapture?.(e.pointerId);
      }}
      onPointerMove={(e) => {
        const from = dragged.current;
        if (!from) return;
        // A release the canvas never heard, outside the window or mid alt-tab, still ends the drag.
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
      {drawn && view && (
        <div
          className="diagram-drawn absolute top-0 left-0 origin-top-left"
          style={{
            width: drawn.size.wide,
            height: drawn.size.tall,
            transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})`,
          }}
          // biome-ignore lint/security/noDangerouslySetInnerHtml: Mermaid draws in strict mode, which leaves no script in the SVG.
          dangerouslySetInnerHTML={{ __html: drawn.svg }}
        />
      )}
      {broken && (
        <p
          role="status"
          className="absolute top-3 left-1/2 -translate-x-1/2 rounded-md bg-panel px-3 py-1.5 text-[12.5px] text-soft shadow"
        >
          {t("diagramUnreadable")}
        </p>
      )}
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
    </section>
  );
}
