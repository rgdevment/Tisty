import { useEffect, useRef } from "react";
import {
  dragging,
  dragTo,
  flipFold,
  fold,
  nudge,
  RAIL,
  stopDragging,
  unwiden,
  WIDEST,
} from "../breadth";
import { t } from "../locales";
import { onMac } from "./WindowChrome";

const STEP = 16;
const STILL = 3;

export const FOLD_KEY = onMac ? "⌘B" : "Ctrl+B";

const writable = (target: EventTarget | null): boolean =>
  target instanceof Element &&
  Boolean(target.closest("input, textarea, select, [contenteditable], .cm-editor"));

const covered = (): boolean => document.querySelector('[aria-modal="true"]') !== null;

export function useFoldKey(shown: boolean) {
  useEffect(() => {
    if (!shown) return;
    const keyed = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.repeat || event.altKey || event.shiftKey) return;
      if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== "b") return;
      if (writable(event.target) || covered()) return;
      event.preventDefault();
      flipFold();
    };
    window.addEventListener("keydown", keyed);
    return () => window.removeEventListener("keydown", keyed);
  }, [shown]);
}

export function FoldButton({ folded }: { folded: boolean }) {
  const label = t(folded ? "sideUnfold" : "sideFold");
  return (
    <button
      type="button"
      onClick={() => fold(!folded)}
      aria-label={label}
      title={`${label} · ${FOLD_KEY}`}
      className="grid size-7 place-items-center rounded-[10px] text-faint transition-colors duration-150 hover:bg-hover hover:text-ink focus-visible:ring-2 focus-visible:ring-accent focus-visible:outline-none"
    >
      <svg
        viewBox="0 0 24 24"
        aria-hidden="true"
        fill="none"
        stroke="currentColor"
        strokeWidth={1.7}
        strokeLinecap="round"
        strokeLinejoin="round"
        className="size-[17px]"
      >
        <rect x="3.5" y="4.5" width="17" height="15" rx="3.5" />
        <path d="M9.5 4.5v15" />
        <path d={folded ? "m13.5 10 2 2-2 2" : "m16 10-2 2 2 2"} />
      </svg>
    </button>
  );
}

type Held = { x: number; width: number; moved: boolean };

const letGo = () => {
  stopDragging();
  document.body.style.cursor = "";
};

export default function SideEdge({ width, folded }: { width: number; folded: boolean }) {
  const from = useRef<Held | null>(null);

  const release = (x: number) => {
    const held = from.current;
    from.current = null;
    letGo();
    if (held?.moved) dragTo(held.width + x - held.x);
  };

  useEffect(
    () => () => {
      if (from.current) letGo();
    },
    [],
  );

  return (
    <hr
      aria-orientation="vertical"
      aria-label={t("sideResize")}
      aria-valuemin={RAIL}
      aria-valuemax={WIDEST}
      aria-valuenow={width}
      aria-valuetext={folded ? t("sideFolded") : `${width} px`}
      tabIndex={0}
      title={t("sideResizeHow")}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        event.currentTarget.setPointerCapture(event.pointerId);
        from.current = { x: event.clientX, width, moved: false };
        document.body.style.cursor = "col-resize";
      }}
      onPointerMove={(event) => {
        const held = from.current;
        if (!held) return;
        if (!held.moved && Math.abs(event.clientX - held.x) < STILL) return;
        held.moved = true;
        dragging(held.width + event.clientX - held.x);
      }}
      onPointerUp={(event) => release(event.clientX)}
      onPointerCancel={(event) => release(event.clientX)}
      onDoubleClick={unwiden}
      onKeyDown={(event) => {
        const by = { ArrowLeft: -STEP, ArrowRight: STEP }[event.key];
        if (by !== undefined) {
          event.preventDefault();
          nudge(width, by);
        } else if (event.key === "Enter") {
          event.preventDefault();
          flipFold();
        }
      }}
      style={{ left: "calc(var(--tisty-side-live, var(--tisty-side)) - 1px)" }}
      className="fixed inset-y-0 z-30 m-0 h-auto w-[9px] cursor-col-resize touch-none border-0 bg-transparent bg-clip-content pr-2 transition-colors duration-150 hover:bg-accent/40 focus-visible:bg-accent focus-visible:outline-none active:bg-accent"
    />
  );
}
