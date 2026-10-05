import { useSyncExternalStore } from "react";
import { AXES, type Axis } from "./archive";

const KEPT = "tisty.grouped";

const kept = (): Axis => {
  try {
    const said = localStorage.getItem(KEPT);
    return AXES.find((one) => one === said) ?? "time";
  } catch {
    return "time";
  }
};

const told = new Set<() => void>();

export const group = (axis: Axis) => {
  try {
    localStorage.setItem(KEPT, axis);
  } catch {}
  for (const one of told) one();
};

const subscribe = (change: () => void) => {
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useGrouped = (): Axis => useSyncExternalStore(subscribe, kept);
