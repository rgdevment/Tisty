import { useSyncExternalStore } from "react";
import { type Machine, waitingMachines } from "./core";

export interface Knocking {
  waiting: Machine[];
  quiet: boolean;
  asking: boolean;
}

let now: Knocking = { waiting: [], quiet: false, asking: false };
const told = new Set<() => void>();

const set = (next: Partial<Knocking>) => {
  now = { ...now, ...next };
  for (const one of told) one();
};

// A machine that was not waiting before brings the card back, even after «Not now».
export const knock = (): Promise<void> =>
  waitingMachines()
    .then((waiting) => {
      const fresh = waiting.some((one) => !now.waiting.some((had) => had.id === one.id));
      set({ waiting, quiet: now.quiet && !fresh });
    })
    .catch(() => undefined);

export const settled = () => set({ waiting: [] });

export const hush = () => set({ quiet: true });

export const ask = () => set({ asking: true });

export const done = () => set({ asking: false });

const subscribe = (change: () => void) => {
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useKnocking = (): Knocking => useSyncExternalStore(subscribe, () => now);

export const shownAs = (one: Machine): string => one.name || one.called;
