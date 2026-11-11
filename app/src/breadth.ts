import { useSyncExternalStore } from "react";
import { stored } from "./stored";

export type Breadth = { width: number | null; folded: boolean };

type Kept = Breadth & { foldedNarrow: boolean };

const ON_MAC = navigator.userAgent.includes("Macintosh");

export const RAIL = ON_MAC ? 76 : 56;
export const NARROWEST = 294;
export const WIDEST = 480;
export const FOLDS_UNDER = NARROWEST;
export const WIDE_WINDOW = 1440;
const AS_SHIPPED_WIDTH = 336;
const AS_SHIPPED_WIDE = 350;
const LIVE = "--tisty-side-live";

const fitted = (width: number): number => Math.round(Math.min(WIDEST, Math.max(NARROWEST, width)));

const kept = stored<Kept>("tisty.breadth", (said) => ({
  width: typeof said.width === "number" ? fitted(said.width) : null,
  folded: said.folded === true,
  foldedNarrow: said.foldedNarrow !== false,
}));

const NARROW = `not all and (min-width: ${WIDE_WINDOW}px)`;

const narrow = (): boolean => window.matchMedia?.(NARROW).matches ?? false;

const watchNarrow = (change: () => void) => {
  const query = window.matchMedia?.(NARROW);
  query?.addEventListener?.("change", change);
  return () => query?.removeEventListener?.("change", change);
};

const seen = (now: Kept, inNarrow: boolean): Breadth => ({
  width: now.width,
  folded: inNarrow ? now.foldedNarrow : now.folded,
});

const current = (): Breadth => seen(kept.current(), narrow());

export const useBreadth = (): Breadth =>
  seen(kept.use(), useSyncExternalStore(watchNarrow, narrow));

const put = (width: number | null, folded: boolean) =>
  kept.put({ ...kept.current(), width, ...(narrow() ? { foldedNarrow: folded } : { folded }) });

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
