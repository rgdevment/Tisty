import { useSyncExternalStore } from "react";
import { added, type Heard, heard, onHeard, type Sum } from "./coming";
import { type Carrying, cloudForget, type Moving, moveTo, type Toward } from "./core";

/** Where a move stands, in the two steps the person sees: bringing home first, then sending on. */
export type MoveAlong =
  | { step: 1; bringing?: { done: number; whole: number } }
  | ({ step: 2 } & Sum);

export type Waited = "limit" | "offline" | "troubled" | "fetching";

export interface Run {
  toward: Toward;
  from?: Carrying["mode"];
  going: boolean;
  result?: Moving;
  failed?: unknown;
  pausedUntil?: number;
  waited?: Waited;
  along?: MoveAlong;
  /** Tries in a row that could not reach the place, and since when, so a blip names no cause. */
  failing?: { rounds: number; since: number };
  /** Bytes still to come down when the wait on the folder's program began, so the bar shows what arrived. */
  fetchingFrom?: number;
  watched: boolean;
}

// A pause Google asks for that ends within minutes is waited out here; a longer one leaves the choice to the person.
const SHORT_PAUSE = 180;
const MARGIN = 2;
const STALLS = 3;
// A large file can take minutes to come down; five with none arriving is long enough to hand the move over.
const FETCH_STALLS = 15;
export const RETRY_IN = 20;

let run: Run | null = null;
let watchers = 0;
let resuming: ReturnType<typeof setTimeout> | undefined;
let stalled = 0;
let lastLeft: number | undefined;
let wasFetching = false;
let fetchStalled = 0;
let fetchingFrom: number | undefined;
let failing: Run["failing"];
// Bumped whenever a run is let go of, so an answer or a timer of the run before never brings it back.
let epoch = 0;
const told = new Set<() => void>();
const afterMoves = new Set<() => void>();
const ups = new Map<number, Sum>();
let upBefore: Sum = { done: 0, whole: 0, sent: 0, weighs: 0 };

const set = (next: Run | null) => {
  run = next && { ...next, fetchingFrom, watched: watchers > 0 };
  for (const one of told) one();
};

// What an earlier try sent stays sent, and what it left is counted again by the try that sends it.
const settleUps = () => {
  const sum = added(ups.values(), upBefore);
  upBefore = { done: sum.done, whole: sum.done, sent: sum.sent, weighs: sum.sent };
  ups.clear();
};

const forgetUps = () => {
  ups.clear();
  upBefore = { done: 0, whole: 0, sent: 0, weighs: 0 };
};

const sentSoFar = (): number => added(ups.values(), upBefore).done;

// Once a move sends, it never goes back to saying it brings, whatever a later try looks over first.
export const alongOf = (step: Heard, was?: MoveAlong): MoveAlong => {
  if (step.stage === "uploading") {
    const up = step.up ?? { part: 0, sent: 0, weighs: 0 };
    ups.set(up.part, { done: step.done, whole: step.whole, sent: up.sent, weighs: up.weighs });
    return { step: 2, ...added(ups.values(), upBefore) };
  }
  if (was?.step === 2) return was;
  if (step.stage === "attachments" && step.whole > 0)
    return { step: 1, bringing: { done: step.done, whole: step.whole } };
  return was ?? { step: 1 };
};

onHeard((step) => {
  if (!run?.going) return;
  set({ ...run, along: alongOf(step, run.along) });
});

const waitOf = (now: Moving): { wait: number; waited: Waited } | undefined => {
  if (now.state !== "waiting") return undefined;
  // A large file the folder's program brings down can take minutes and leave nothing new between tries.
  if (now.reason === "fetching")
    return fetchStalled >= FETCH_STALLS ? undefined : { wait: RETRY_IN, waited: "fetching" };
  if (stalled >= STALLS) return undefined;
  if (now.reason === "limit" && typeof now.wait === "number" && now.wait <= SHORT_PAUSE)
    return { wait: now.wait + MARGIN, waited: "limit" };
  if (now.reason === "offline" || now.reason === "troubled")
    return { wait: RETRY_IN, waited: now.reason };
  return undefined;
};

const go = (toward: Toward, from?: Carrying["mode"], along?: MoveAlong) => {
  if (run?.going) return;
  clearTimeout(resuming);
  const mine = epoch;
  heard(null);
  settleUps();
  const before = sentSoFar();
  const began = Math.floor(Date.now() / 1000);
  // Looking again while the folder's program brings files down still says what it waits for.
  const fetching = run?.waited === "fetching";
  set({
    toward,
    from,
    going: true,
    along,
    watched: false,
    ...(fetching ? { waited: "fetching", result: run?.result } : {}),
  });
  moveTo(toward)
    .then((now) => {
      if (mine !== epoch) return;
      heard(null);
      failing =
        now.state === "waiting" && now.reason === "offline"
          ? { rounds: (failing?.rounds ?? 0) + 1, since: failing?.since ?? began }
          : undefined;
      if (now.state === "done") for (const one of afterMoves) one();
      // Files still coming down put the move back at its first step, with nothing yet to count as brought.
      const along: MoveAlong | undefined =
        now.state === "waiting" && now.reason === "fetching" ? { step: 1 } : run?.along;
      const left = now.state === "waiting" ? now.left.files : undefined;
      // Downloads and other waits count files apart, so each keeps its own tally of tries that moved nothing.
      const fetching = now.state === "waiting" && now.reason === "fetching";
      const switched = lastLeft !== undefined && fetching !== wasFetching;
      const freshWait = fetching && !wasFetching;
      wasFetching = fetching;
      const moved =
        lastLeft === undefined || switched
          ? sentSoFar() > before
          : left !== undefined && left < lastLeft;
      if (moved) {
        stalled = 0;
        fetchStalled = 0;
      } else if (left !== undefined && lastLeft !== undefined) {
        if (fetching) fetchStalled += 1;
        else stalled += 1;
      }
      lastLeft = left;
      const bytes = now.state === "waiting" ? now.left.bytes : 0;
      if (fetching) fetchingFrom = freshWait ? bytes : Math.max(fetchingFrom ?? 0, bytes);
      const waits = waitOf(now);
      if (waits === undefined) {
        set({ toward, from, going: false, result: now, along, failing, watched: false });
        return;
      }
      const until = Date.now() + waits.wait * 1000;
      set({
        toward,
        from,
        going: false,
        result: now,
        pausedUntil: until,
        waited: waits.waited,
        along,
        failing,
        watched: false,
      });
      resuming = setTimeout(() => {
        if (mine === epoch) go(toward, from, run?.along);
      }, until - Date.now());
    })
    .catch((failed) => {
      if (mine !== epoch) return;
      heard(null);
      set({ toward, from, going: false, failed: failed ?? {}, watched: false });
    });
};

/** A move that waits goes on from where it stood when the person carries on; any other starts afresh. */
export const startMove = (toward: Toward, from?: Carrying["mode"]) => {
  if (run?.going) return;
  stalled = 0;
  fetchStalled = 0;
  const goesOn = run?.result?.state === "waiting";
  if (!goesOn) {
    lastLeft = undefined;
    fetchingFrom = undefined;
    failing = undefined;
    forgetUps();
  }
  go(toward, from ?? run?.from, goesOn ? run?.along : undefined);
};

export const endMove = () => {
  epoch += 1;
  clearTimeout(resuming);
  stalled = 0;
  fetchStalled = 0;
  lastLeft = undefined;
  wasFetching = false;
  fetchingFrom = undefined;
  failing = undefined;
  forgetUps();
  set(null);
};

/// Put away once seen through: the access to a cloud left behind is let go of unless the person settled it already.
export const dismissMove = (forget = true) => {
  const done = run?.result?.state === "done" ? run.result : undefined;
  if (forget && done?.told && done.leftCloud)
    void cloudForget(done.leftCloud.account, done.leftCloud.provider).catch(() => undefined);
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
