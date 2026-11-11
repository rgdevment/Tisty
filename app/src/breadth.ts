import { useLayoutEffect, useSyncExternalStore } from "react";
import { stored } from "./stored";

export type Breadth = { width: number | null; folded: boolean };

type Sway = "free" | "giving" | "held";

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

const watch = (change: () => void) => {
  const resized = () => {
    if (!crowded()) room.held = false;
    change();
  };
  heard.add(change);
  window.addEventListener("resize", resized);
  return () => {
    heard.delete(change);
    window.removeEventListener("resize", resized);
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
  kept.put({ ...kept.current(), width });
  room.held = !folded;
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
