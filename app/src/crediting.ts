import { useSyncExternalStore } from "react";

export type Credit = { rows: boolean; inside: boolean };

const KEPT = "tisty.credit";
const AS_SHIPPED: Credit = { rows: false, inside: true };

const kept = (): Credit => {
  try {
    const said = (JSON.parse(localStorage.getItem(KEPT) ?? "{}") ?? {}) as Record<string, unknown>;
    return {
      rows: typeof said.rows === "boolean" ? said.rows : AS_SHIPPED.rows,
      inside: typeof said.inside === "boolean" ? said.inside : AS_SHIPPED.inside,
    };
  } catch {
    return AS_SHIPPED;
  }
};

const told = new Set<() => void>();
let now: Credit | null = null;

const current = (): Credit => {
  now ??= kept();
  return now;
};

export const credit = (where: keyof Credit, named: boolean) => {
  now = { ...current(), [where]: named };
  try {
    localStorage.setItem(KEPT, JSON.stringify(now));
  } catch {}
  for (const one of told) one();
};

const subscribe = (change: () => void) => {
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useCredit = (): Credit => useSyncExternalStore(subscribe, current);
