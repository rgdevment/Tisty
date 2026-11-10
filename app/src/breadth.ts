import { stored } from "./stored";

export type Breadth = { width: number | null; folded: boolean };

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

const kept = stored<Breadth>("tisty.breadth", (said) => ({
  width: typeof said.width === "number" ? fitted(said.width) : null,
  folded: said.folded === true,
}));

export const useBreadth = kept.use;

export const fold = (folded: boolean) => kept.put({ ...kept.current(), folded });

export const flipFold = () => fold(!kept.current().folded);

export const unwiden = () => kept.put({ width: null, folded: false });

export const nudge = (from: number, by: number) => {
  const was = kept.current();
  if (was.folded) {
    if (by > 0) fold(false);
    return;
  }
  if (by < 0 && from <= NARROWEST) return fold(true);
  kept.put({ width: fitted(from + by), folded: false });
};

export const dragTo = (width: number) =>
  kept.put(
    width < FOLDS_UNDER
      ? { ...kept.current(), folded: true }
      : { width: fitted(width), folded: false },
  );

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
