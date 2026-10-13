import { useSyncExternalStore } from "react";
import { heard } from "./coming";
import { cloudForget, type Moving, moveTo, type Toward } from "./core";

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
const STALLS = 3;

let run: Run | null = null;
let watchers = 0;
let resuming: ReturnType<typeof setTimeout> | undefined;
let stalled = 0;
let lastLeft: number | undefined;
// Bumped whenever a run is let go of, so an answer or a timer of the run before never brings it back.
let epoch = 0;
const told = new Set<() => void>();
const afterMoves = new Set<() => void>();

const set = (next: Run | null) => {
  run = next && { ...next, watched: watchers > 0 };
  for (const one of told) one();
};

const shortPause = (now: Moving): number | undefined =>
  stalled < STALLS &&
  now.state === "waiting" &&
  now.reason === "limit" &&
  typeof now.wait === "number" &&
  now.wait <= SHORT_PAUSE
    ? now.wait
    : undefined;

const go = (toward: Toward) => {
  if (run?.going) return;
  clearTimeout(resuming);
  const mine = epoch;
  heard(null);
  set({ toward, going: true, watched: false });
  moveTo(toward)
    .then((now) => {
      if (mine !== epoch) return;
      heard(null);
      if (now.state === "done") for (const one of afterMoves) one();
      const left = now.state === "waiting" ? now.left.files : undefined;
      stalled = left !== undefined && lastLeft !== undefined && left >= lastLeft ? stalled + 1 : 0;
      lastLeft = left;
      const wait = shortPause(now);
      if (wait === undefined) {
        set({ toward, going: false, result: now, watched: false });
        return;
      }
      const until = Date.now() + (wait + MARGIN) * 1000;
      set({ toward, going: false, result: now, pausedUntil: until, watched: false });
      resuming = setTimeout(() => {
        if (mine === epoch) go(toward);
      }, until - Date.now());
    })
    .catch((failed) => {
      if (mine !== epoch) return;
      heard(null);
      set({ toward, going: false, failed: failed ?? {}, watched: false });
    });
};

export const startMove = (toward: Toward) => {
  if (run?.going) return;
  stalled = 0;
  lastLeft = undefined;
  go(toward);
};

export const endMove = () => {
  epoch += 1;
  clearTimeout(resuming);
  stalled = 0;
  lastLeft = undefined;
  set(null);
};

/// Put away once seen through: the access to a cloud left behind is let go of unless the person settled it already.
export const dismissMove = (forget = true) => {
  const done = run?.result?.state === "done" ? run.result : undefined;
  if (forget && done?.told && done.leftCloud)
    void cloudForget(done.leftCloud.account).catch(() => undefined);
  endMove();
};

export const afterMove = (then: () => void) => {
  afterMoves.add(then);
  return () => {
    afterMoves.delete(then);
  };
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
