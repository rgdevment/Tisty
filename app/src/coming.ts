import { listen } from "@tauri-apps/api/event";
import { useSyncExternalStore } from "react";
import { saidPlainly } from "./refusal";

export type Stage = "log" | "papers" | "attachments";

interface Heard {
  stage: Stage;
  done: number;
  whole: number;
  joining?: boolean;
}

export interface Coming extends Heard {
  since: number;
}

let now: Coming | null = null;
let stuck: string | null = null;
let hearing = false;
const told = new Set<() => void>();

const tell = () => {
  for (const one of told) one();
};

export const heard = (step: Heard | null, at = Date.now()) => {
  now = step ? { ...step, since: now?.since ?? at } : null;
  tell();
};

export const ended = (why: unknown) => {
  now = null;
  stuck = why ? saidPlainly(why) : null;
  tell();
};

const hear = () => {
  if (hearing) return;
  hearing = true;
  void listen<Heard>("bringing", (step) => heard(step.payload));
  void listen<unknown>("brought", (end) => ended(end.payload));
};

const subscribe = (change: () => void) => {
  hear();
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useComing = (): Coming | null => useSyncExternalStore(subscribe, () => now);

export const useStuck = (): string | null => useSyncExternalStore(subscribe, () => stuck);
