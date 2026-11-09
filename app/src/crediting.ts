import { useSyncExternalStore } from "react";

export type Credit = { rows: boolean; inside: boolean; mark: boolean };
export type Signing = "named" | "agent" | "mark";

const KEPT = "tisty.credit";
const AS_SHIPPED: Credit = { rows: false, inside: true, mark: false };

const kept = (): Credit => {
  try {
    const said = (JSON.parse(localStorage.getItem(KEPT) ?? "{}") ?? {}) as Record<string, unknown>;
    const chosen = (key: keyof Credit) =>
      typeof said[key] === "boolean" ? (said[key] as boolean) : AS_SHIPPED[key];
    return { rows: chosen("rows"), inside: chosen("inside"), mark: chosen("mark") };
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

export const credit = (where: keyof Credit, on: boolean) => {
  now = { ...current(), [where]: on };
  try {
    localStorage.setItem(KEPT, JSON.stringify(now));
  } catch {}
  for (const one of told) one();
};

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

export const useCredit = (): Credit => useSyncExternalStore(subscribe, current);

export const signing = (chosen: Credit, where: "rows" | "inside"): Signing => {
  if (chosen.mark) return "mark";
  return chosen[where] ? "named" : "agent";
};

export const useSigning = (where: "rows" | "inside"): Signing => signing(useCredit(), where);
