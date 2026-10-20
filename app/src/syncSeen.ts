import { useSyncExternalStore } from "react";
import { type Carrying, type CloudStatus, cloudStatus, syncState } from "./core";

export interface Seen {
  state: Carrying | null;
  status: CloudStatus | null;
}

let seen: Seen = { state: null, status: null };
let asking: Promise<void> | null = null;
let again = false;
const watching = new Set<() => void>();

const read = async () => {
  const state = (await syncState()) ?? null;
  const status = state?.mode === "cloud" ? ((await cloudStatus().catch(() => null)) ?? null) : null;
  seen = { state, status };
  for (const one of watching) one();
};

// A look asked for while one is out may come after what that one read, so it runs once more.
export const lookAgain = (): Promise<void> => {
  if (asking) {
    again = true;
    return asking;
  }
  asking = read()
    .catch(() => undefined)
    .finally(() => {
      asking = null;
      if (again) {
        again = false;
        void lookAgain();
      }
    });
  return asking;
};

const subscribe = (change: () => void) => {
  watching.add(change);
  return () => {
    watching.delete(change);
  };
};

export const useSyncSeen = (): Seen => useSyncExternalStore(subscribe, () => seen);
