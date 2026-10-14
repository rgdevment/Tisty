import { listen } from "@tauri-apps/api/event";
import { useSyncExternalStore } from "react";

export type Stage = "log" | "papers" | "attachments" | "uploading";

/** One part of a round that sends, told on its own: the window adds the parts up. */
export interface Up {
  part: number;
  sent: number;
  weighs: number;
}

export interface Heard {
  stage: Stage;
  done: number;
  whole: number;
  joining?: boolean;
  up?: Up;
}

export interface Coming extends Heard {
  since: number;
  sent?: number;
  weighs?: number;
}

export interface Sum {
  done: number;
  whole: number;
  sent: number;
  weighs: number;
}

let now: Coming | null = null;
let stuck: unknown = null;
let carded: string | null = null;
let choosing = false;
let hearing = false;
const told = new Set<() => void>();
const hearers = new Set<(step: Heard) => void>();
const parts = new Map<number, Sum>();

const tell = () => {
  for (const one of told) one();
};

export const added = (
  all: Iterable<Sum>,
  from: Sum = { done: 0, whole: 0, sent: 0, weighs: 0 },
) => {
  const sum = { ...from };
  for (const one of all) {
    sum.done += one.done;
    sum.whole += one.whole;
    sum.sent += one.sent;
    sum.weighs += one.weighs;
  }
  return sum;
};

const summed = (step: Heard): Heard & Partial<Sum> => {
  if (!step.up) return step;
  const { part, sent, weighs } = step.up;
  parts.set(part, { done: step.done, whole: step.whole, sent, weighs });
  return { ...step, ...added(parts.values()) };
};

export const onHeard = (hear: (step: Heard) => void) => {
  hearers.add(hear);
  return () => {
    hearers.delete(hear);
  };
};

export const heard = (step: Heard | null, at = Date.now()) => {
  if (!step) parts.clear();
  now = step ? { ...summed(step), since: now?.since ?? at } : null;
  if (step) for (const one of hearers) one(step);
  tell();
};

export const ended = (why: unknown) => {
  now = null;
  parts.clear();
  stuck = why ?? null;
  tell();
};

/** The trouble a card in Settings already shows with its own way out, so it is not said again elsewhere. */
export const carding = (code: string | null) => {
  if (carded === code) return;
  carded = code;
  tell();
};

export const choose = (now: boolean) => {
  if (choosing === now) return;
  choosing = now;
  tell();
};

const APART = new Set(["wouldReset", "otherStore", "restoredApart"]);

export const apartWaits = (said: unknown): boolean =>
  APART.has((said as { code?: string } | null)?.code ?? "");

export const unstuck = () => {
  stuck = null;
  tell();
};

const hear = () => {
  if (hearing) return;
  hearing = true;
  void listen<Heard>("bringing", (step) => heard(step.payload));
  void listen<unknown>("brought", (end) => ended(end.payload));
  void listen("unstuck", unstuck);
};

const subscribe = (change: () => void) => {
  hear();
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useComing = (): Coming | null => useSyncExternalStore(subscribe, () => now);

export const useStuck = (): unknown => useSyncExternalStore(subscribe, () => stuck);

export const useCarded = (): string | null => useSyncExternalStore(subscribe, () => carded);

export const useChoosing = (): boolean => useSyncExternalStore(subscribe, () => choosing);

export const useAwaitsChoice = (): boolean => {
  const open = useChoosing();
  const told = useStuck();
  return open || apartWaits(told);
};
