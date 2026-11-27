import { useSyncExternalStore } from "react";

export type Freshness = {
  readonly rounds: () => number;
  readonly heard: (listener: () => void) => () => void;
  readonly landed: () => void;
};

// Only what reads the rounds is drawn again when one lands, not the whole window around it.
export const freshness = (): Freshness => {
  let rounds = 0;
  const listeners = new Set<() => void>();
  return {
    rounds: () => rounds,
    heard: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    landed: () => {
      rounds += 1;
      for (const listener of listeners) listener();
    },
  };
};

export const useRounds = (fresh: Freshness): number =>
  useSyncExternalStore(fresh.heard, fresh.rounds);
