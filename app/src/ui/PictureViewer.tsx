import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { fill, t } from "../locales";
import type { Shot } from "./paging";

const MOST = 6;
const LEAST = 1;

interface Props {
  shots: Shot[];
  start: number;
  onClose: () => void;
  onSave?: (kept: string) => void;
  onOpen?: (kept: string) => void;
}

const bar =
  "rounded-md px-2.5 py-1 text-[12.5px] text-white/70 hover:bg-white/10 hover:text-white disabled:opacity-40";

export default function PictureViewer({ shots, start, onClose, onSave, onOpen }: Props) {
  const [nth, setNth] = useState(start);
  const [zoom, setZoom] = useState(1);
  const [shift, setShift] = useState({ x: 0, y: 0 });
  const dragging = useRef<{ x: number; y: number } | null>(null);
  const shot = shots[nth];

  const reset = () => {
    setZoom(1);
    setShift({ x: 0, y: 0 });
  };
  const go = (step: number) => {
    if (shots.length < 2) return;
    setNth((now) => (now + step + shots.length) % shots.length);
    reset();
  };
  const zoomed = (by: number) =>
    setZoom((now) => {
      const next = Math.min(MOST, Math.max(LEAST, now * by));
      if (next === LEAST) setShift({ x: 0, y: 0 });
      return next;
    });

  // biome-ignore lint/correctness/useExhaustiveDependencies: the keys read the state as it stands when pressed
  useEffect(() => {
    const keyed = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
      else if (event.key === "ArrowRight") go(1);
      else if (event.key === "ArrowLeft") go(-1);
      else if (event.key === "+" || event.key === "=") zoomed(1.25);
      else if (event.key === "-") zoomed(0.8);
      else if (event.key === "0") reset();
      else return;
      event.preventDefault();
    };
    window.addEventListener("keydown", keyed);
    return () => window.removeEventListener("keydown", keyed);
  }, [shots.length]);

  if (!shot) return null;

  return createPortal(
    <div
      role="dialog"
      aria-modal="true"
      aria-label={shot.alt || t("pictureLook")}
      className="fixed inset-0 z-50 flex flex-col bg-black/70 backdrop-blur-sm"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div className="flex items-center gap-1 px-3 py-2">
        <span className="min-w-0 flex-1 truncate px-2 text-[12.5px] text-white/70">
          {shot.alt}
          {shots.length > 1 && ` · ${fill("pictureCount", String(nth + 1), String(shots.length))}`}
        </span>
        <button
          type="button"
          className={bar}
          onClick={() => zoomed(0.8)}
          aria-label={t("pictureLess")}
        >
          −
        </button>
        <button type="button" className={bar} onClick={reset}>
          {Math.round(zoom * 100)}%
        </button>
        <button
          type="button"
          className={bar}
          onClick={() => zoomed(1.25)}
          aria-label={t("pictureMore")}
        >
          +
        </button>
        {shot.kept && onSave && (
          <button type="button" className={bar} onClick={() => shot.kept && onSave(shot.kept)}>
            {t("pictureSave")}
          </button>
        )}
        {shot.kept && onOpen && (
          <button type="button" className={bar} onClick={() => shot.kept && onOpen(shot.kept)}>
            {t("pictureOpen")}
          </button>
        )}
        <button type="button" className={bar} onClick={onClose} aria-label={t("pictureClose")}>
          ✕
        </button>
      </div>

      <div
        className="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden"
        onMouseDown={(event) => {
          if (event.target === event.currentTarget) onClose();
        }}
        onWheel={(event) => zoomed(event.deltaY < 0 ? 1.15 : 0.87)}
      >
        {shots.length > 1 && (
          <button
            type="button"
            onClick={() => go(-1)}
            aria-label={t("picturePrevious")}
            className={`${bar} absolute left-3 z-10 text-[21px]`}
          >
            ‹
          </button>
        )}
        {/* biome-ignore lint/a11y/noNoninteractiveElementInteractions: dragging moves a zoomed picture; the keys do the rest */}
        <img
          src={shot.src}
          alt={shot.alt}
          draggable={false}
          onDoubleClick={() => (zoom > 1 ? reset() : setZoom(2))}
          onMouseDown={(event) => {
            if (zoom <= 1) return;
            event.preventDefault();
            dragging.current = { x: event.clientX - shift.x, y: event.clientY - shift.y };
          }}
          onMouseMove={(event) => {
            const from = dragging.current;
            if (from) setShift({ x: event.clientX - from.x, y: event.clientY - from.y });
          }}
          onMouseUp={() => {
            dragging.current = null;
          }}
          onMouseLeave={() => {
            dragging.current = null;
          }}
          style={{ transform: `translate(${shift.x}px, ${shift.y}px) scale(${zoom})` }}
          className={`max-h-[88vh] max-w-[92vw] rounded-md bg-white object-contain transition-transform duration-100 motion-reduce:transition-none ${
            zoom > 1 ? "cursor-grab active:cursor-grabbing" : "cursor-zoom-in"
          }`}
        />
        {shots.length > 1 && (
          <button
            type="button"
            onClick={() => go(1)}
            aria-label={t("pictureNext")}
            className={`${bar} absolute right-3 z-10 text-[21px]`}
          >
            ›
          </button>
        )}
      </div>
    </div>,
    document.body,
  );
}
