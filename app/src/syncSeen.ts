import { useSyncExternalStore } from "react";
import { type Carrying, type CloudStatus, cloudStatus, syncState } from "./core";

interface Seen {
  state: Carrying | null;
  status: CloudStatus | null;
}

let seen: Seen = { state: null, status: null };
let asking: Promise<Seen> | null = null;
let again = false;
const watching = new Set<() => void>();

// A cloud status that cannot be read keeps the last one, so a pause or a lost access is not forgotten in between.
const read = async (): Promise<Seen> => {
  const state = (await syncState()) ?? null;
  const status = state?.mode === "cloud" ? await cloudStatus().catch(() => seen.status) : null;
  seen = { state, status };
  for (const one of watching) one();
  return seen;
};

// A look asked for while one is out may come after what that one read, so it runs once more.
export const refresh = (): Promise<Seen> => {
  if (asking) {
    again = true;
    return asking;
  }
  asking = read().finally(() => {
    asking = null;
    if (again) {
      again = false;
      void lookAgain();
    }
  });
  return asking;
};

export const lookAgain = (): Promise<void> =>
  refresh().then(
    () => undefined,
    () => undefined,
  );

const subscribe = (change: () => void) => {
  watching.add(change);
  return () => {
    watching.delete(change);
  };
};

export const useSyncSeen = (): Seen => useSyncExternalStore(subscribe, () => seen);
