import { useSyncExternalStore } from "react";
import { type Moving, moveTo, type Toward } from "./core";

export interface Run {
  toward: Toward;
  going: boolean;
  result?: Moving;
  failed?: unknown;
  pausedUntil?: number;
  watched: boolean;
}

// A pause Google asks for that ends within minutes is waited out here; a longer one leaves the choice to the person.
const SHORT_PAUSE = 180;
const MARGIN = 2;

let run: Run | null = null;
let watchers = 0;
let resuming: ReturnType<typeof setTimeout> | undefined;
const told = new Set<() => void>();

const set = (next: Run | null) => {
  run = next && { ...next, watched: watchers > 0 };
  for (const one of told) one();
};

const shortPause = (now: Moving): number | undefined =>
  now.state === "waiting" &&
  now.reason === "limit" &&
  typeof now.wait === "number" &&
  now.wait <= SHORT_PAUSE
    ? now.wait
    : undefined;

export const startMove = (toward: Toward) => {
  if (run?.going) return;
  clearTimeout(resuming);
  set({ toward, going: true, watched: false });
  moveTo(toward)
    .then((now) => {
      const wait = shortPause(now);
      if (wait === undefined) {
        set({ toward, going: false, result: now, watched: false });
        return;
      }
      const until = Date.now() + (wait + MARGIN) * 1000;
      set({ toward, going: false, result: now, pausedUntil: until, watched: false });
      resuming = setTimeout(() => startMove(toward), until - Date.now());
    })
    .catch((failed) => set({ toward, going: false, failed: failed ?? {}, watched: false }));
};

export const endMove = () => {
  clearTimeout(resuming);
  set(null);
};

export const watchMove = () => {
  watchers += 1;
  if (run) set(run);
  return () => {
    watchers -= 1;
    if (run) set(run);
  };
};

const subscribe = (change: () => void) => {
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useMove = (): Run | null => useSyncExternalStore(subscribe, () => run);

export const underway = (one: Run): boolean => one.going || one.pausedUntil !== undefined;
