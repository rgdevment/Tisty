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

const sameCloud = (one: Carrying | null, other: Carrying | null) =>
  one?.mode === "cloud" &&
  other?.mode === "cloud" &&
  one.cloud === other.cloud &&
  one.cloudProvider === other.cloudProvider;

// A status that cannot be read keeps the last one of the same account, so a pause or a lost access is not forgotten.
const read = async (): Promise<Seen> => {
  const state = (await syncState()) ?? null;
  const kept = sameCloud(seen.state, state) ? seen.status : null;
  const status = state?.mode === "cloud" ? await cloudStatus().catch(() => kept) : null;
  if (
    JSON.stringify(state) === JSON.stringify(seen.state) &&
    JSON.stringify(status) === JSON.stringify(seen.status)
  )
    return seen;
  seen = { state, status };
  for (const one of watching) one();
  return seen;
};

const start = (): Promise<Seen> => {
  const now: Promise<Seen> = read().finally(() => {
    if (asking === now) asking = null;
    if (again) {
      again = false;
      void lookAgain();
    }
  });
  asking = now;
  return now;
};

// A look asked for while one is out may come after what that one read, so it runs once more.
export const lookAgain = (): Promise<void> => {
  if (asking) again = true;
  return (asking ?? start()).then(
    () => undefined,
    () => undefined,
  );
};

// Read after the call, never handed one that started before it.
export const refresh = (): Promise<Seen> => {
  if (!asking) return start();
  const after = () => asking ?? start();
  return asking.then(after, after);
};

const subscribe = (change: () => void) => {
  watching.add(change);
  return () => {
    watching.delete(change);
  };
};

export const useSyncSeen = (): Seen => useSyncExternalStore(subscribe, () => seen);
