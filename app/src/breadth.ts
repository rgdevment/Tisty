import { useLayoutEffect, useSyncExternalStore } from "react";
import { stored } from "./stored";
import { onMac } from "./ui/macKeys";

export type Breadth = { width: number | null; folded: boolean };

type Sway = "free" | "giving" | "held";

export const RAIL = onMac ? 76 : 56;
export const NARROWEST = 294;
export const WIDEST = 480;
export const FOLDS_UNDER = NARROWEST;
export const WIDE_WINDOW = 1440;
const AS_SHIPPED_WIDTH = 336;
const AS_SHIPPED_WIDE = 350;
const LIVE = "--tisty-side-live";

const fitted = (width: number): number => Math.round(Math.min(WIDEST, Math.max(NARROWEST, width)));

const GUTTER = 8;

const kept = stored<Breadth>("tisty.breadth", (said) => ({
  width: typeof said.width === "number" ? fitted(said.width) : null,
  folded: said.folded === true,
}));

const room = { need: 0, held: false };
const heard = new Set<() => void>();
const tell = () => {
  for (const one of heard) one();
};

export const givesWay = (across: number, need: number, width: number | null): boolean => {
  if (!need) return false;
  const open = width ?? (across >= WIDE_WINDOW ? AS_SHIPPED_WIDE : AS_SHIPPED_WIDTH);
  return across - open - GUTTER < need && across - RAIL - GUTTER >= need;
};

const crowded = () => givesWay(window.innerWidth, room.need, kept.current().width);

const sway = (): Sway => (crowded() ? (room.held ? "held" : "giving") : "free");

const settle = () => {
  if (!crowded()) room.held = false;
};

const resized = () => {
  settle();
  tell();
};

const watch = (change: () => void) => {
  if (!heard.size) window.addEventListener("resize", resized);
  heard.add(change);
  return () => {
    heard.delete(change);
    if (!heard.size) window.removeEventListener("resize", resized);
  };
};

const seen = (now: Breadth, how: Sway): Breadth => ({
  width: now.width,
  folded: how === "free" ? now.folded : how === "giving",
});

const current = (): Breadth => seen(kept.current(), sway());

export const useBreadth = (): Breadth => seen(kept.use(), useSyncExternalStore(watch, sway));

export const useRoomFor = (need: number) =>
  useLayoutEffect(() => {
    if (room.need === need) return;
    room.need = need;
    room.held = false;
    tell();
  }, [need]);

const put = (width: number | null, folded: boolean) => {
  if (!crowded()) return kept.put({ width, folded });
  room.held = !folded;
  kept.put({ ...kept.current(), width });
  settle();
  tell();
};

export const fold = (folded: boolean) => put(kept.current().width, folded);

export const flipFold = () => fold(!current().folded);

export const unwiden = () => put(null, false);

export const nudge = (from: number, by: number) => {
  if (current().folded) {
    if (by > 0) fold(false);
    return;
  }
  if (by < 0 && from <= NARROWEST) return fold(true);
  put(fitted(from + by), false);
};

export const dragTo = (width: number) =>
  width < FOLDS_UNDER ? fold(true) : put(fitted(width), false);

export const dragging = (width: number) => {
  const root = document.documentElement;
  root.style.setProperty(LIVE, `${fitted(width)}px`);
  root.dataset.resizing = "";
};

export const stopDragging = () => {
  const root = document.documentElement;
  root.style.removeProperty(LIVE);
  delete root.dataset.resizing;
};

export const railed = (breadth: Breadth, keeping: boolean): Breadth =>
  keeping && breadth.folded ? { ...breadth, folded: false } : breadth;

export const sideWidth = (breadth: Breadth, wide: boolean): number => {
  if (breadth.folded) return RAIL;
  return breadth.width ?? (wide ? AS_SHIPPED_WIDE : AS_SHIPPED_WIDTH);
};

export const chosenWidth = (breadth: Breadth): number | null =>
  breadth.folded ? RAIL : breadth.width;

export type Measure = "narrow" | "normal" | "wide" | "full";

export const MEASURES: readonly Measure[] = ["narrow", "normal", "wide", "full"];

// The default fills the window up to a line that still reads in one sweep of the eye.
export const SHOWN_MEASURE: Measure = "normal";

const isMeasure = (said: unknown): said is Measure =>
  typeof said === "string" && (MEASURES as readonly string[]).includes(said);

// Kept by the computer, never in the document: how wide it reads is the reader's, not its words'.
const measured = stored<Record<string, Measure>>("tisty.measure", (said) =>
  Object.fromEntries(
    Object.entries(said).filter((pair): pair is [string, Measure] => isMeasure(pair[1])),
  ),
);

export const measureOf = (doc: string, kept: Record<string, Measure>): Measure =>
  kept[doc] ?? SHOWN_MEASURE;

export const measureNow = (doc: string): Measure => measureOf(doc, measured.current());

export const useMeasure = (doc: string | undefined): Measure => {
  const kept = measured.use();
  return doc ? measureOf(doc, kept) : SHOWN_MEASURE;
};

export const measureFor = (doc: string, measure: Measure) => {
  const { [doc]: _was, ...others } = measured.current();
  measured.put(measure === SHOWN_MEASURE ? others : { ...others, [doc]: measure });
};
