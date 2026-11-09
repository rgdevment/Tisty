import { useSyncExternalStore } from "react";

export type Breadth = { width: number | null; folded: boolean; moving?: boolean };

export const RAIL = 56;
export const NARROWEST = 220;
export const WIDEST = 480;
export const FOLDS_UNDER = 160;
export const WIDE_WINDOW = 1440;
const AS_SHIPPED_WIDTH = 336;
const AS_SHIPPED_WIDE = 350;

const KEPT = "tisty.breadth";
const AS_SHIPPED: Breadth = { width: null, folded: false };

const fitted = (width: number): number => Math.round(Math.min(WIDEST, Math.max(NARROWEST, width)));

const kept = (): Breadth => {
  try {
    const said = (JSON.parse(localStorage.getItem(KEPT) ?? "{}") ?? {}) as Record<string, unknown>;
    return {
      width: typeof said.width === "number" ? fitted(said.width) : null,
      folded: said.folded === true,
    };
  } catch {
    return AS_SHIPPED;
  }
};

const told = new Set<() => void>();
let now: Breadth | null = null;

const current = (): Breadth => {
  now ??= kept();
  return now;
};

const settle = (next: Breadth, lasting: boolean) => {
  now = { ...next, moving: !lasting };
  if (lasting) {
    try {
      localStorage.setItem(KEPT, JSON.stringify({ width: next.width, folded: next.folded }));
    } catch {}
  }
  for (const one of told) one();
};

export const fold = (folded: boolean) => settle({ ...current(), folded }, true);

export const flipFold = () => fold(!current().folded);

export const unwiden = () => settle(AS_SHIPPED, true);

export const widen = (by: number) =>
  settle({ width: fitted((current().width ?? AS_SHIPPED_WIDTH) + by), folded: false }, true);

export const dragTo = (width: number, lasting: boolean) =>
  settle(
    width < FOLDS_UNDER ? { ...current(), folded: true } : { width: fitted(width), folded: false },
    lasting,
  );

const subscribe = (change: () => void) => {
  const elsewhere = (seen: StorageEvent) => {
    if (seen.key !== KEPT) return;
    now = null;
    change();
  };
  told.add(change);
  window.addEventListener("storage", elsewhere);
  return () => {
    told.delete(change);
    window.removeEventListener("storage", elsewhere);
  };
};

export const useBreadth = (): Breadth => useSyncExternalStore(subscribe, current);

export const railed = (breadth: Breadth, keeping: boolean): Breadth =>
  keeping && breadth.folded ? { ...breadth, folded: false } : breadth;

export const sideWidth = (breadth: Breadth, wide: boolean): number => {
  if (breadth.folded) return RAIL;
  return breadth.width ?? (wide ? AS_SHIPPED_WIDE : AS_SHIPPED_WIDTH);
};
