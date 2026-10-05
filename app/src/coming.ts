import { listen } from "@tauri-apps/api/event";
import { useSyncExternalStore } from "react";

export type Stage = "log" | "papers" | "attachments";

export interface Coming {
  stage: Stage;
  done: number;
  whole: number;
  since: number;
}

interface Heard {
  stage: Stage;
  done: number;
  whole: number;
}

let now: Coming | null = null;
let hearing = false;
const told = new Set<() => void>();

const tell = () => {
  for (const one of told) one();
};

export const heard = (step: Heard | null, at = Date.now()) => {
  now = step ? { ...step, since: now?.since ?? at } : null;
  tell();
};

const hear = () => {
  if (hearing) return;
  hearing = true;
  void listen<Heard>("bringing", (step) => heard(step.payload));
  void listen("brought", () => heard(null));
};

const subscribe = (change: () => void) => {
  hear();
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useComing = (): Coming | null => useSyncExternalStore(subscribe, () => now);
