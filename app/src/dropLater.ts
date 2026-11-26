import { useSyncExternalStore } from "react";
import { holds, queued } from "./saving";

export const UNDO_FOR = 10_000;

export interface Going {
  key: string;
  /// The documents and folders the sidebar stops showing while the delete waits.
  hides: readonly string[];
  /// A folder whose contents go up or elsewhere: until it runs, they are drawn where they will land.
  moves?: { from: string; to: string | null };
  said: string;
  run: () => Promise<void>;
}

const told = new Set<() => void>();
const timers = new Map<string, ReturnType<typeof setTimeout>>();
let waiting: readonly Going[] = [];
let made = 0;

const changed = (next: readonly Going[]) => {
  waiting = next;
  for (const one of told) one();
};

const taken = (key: string): Going | undefined => {
  const one = waiting.find((going) => going.key === key);
  clearTimeout(timers.get(key));
  timers.delete(key);
  if (one) changed(waiting.filter((going) => going !== one));
  return one;
};

// The delete itself goes in the line of writes the window waits for when it closes.
const fire = (key: string) => {
  const one = taken(key);
  if (one) void queued("dropLater", one.run).catch(() => {});
};

export const fireAll = () => {
  for (const one of [...waiting]) fire(one.key);
};

holds(fireAll, { holding: () => waiting.length > 0 });

export const dropLater = (going: Omit<Going, "key">): string => {
  made += 1;
  const key = `drop-${made}`;
  changed([...waiting, { ...going, key }]);
  timers.set(
    key,
    setTimeout(() => fire(key), UNDO_FOR),
  );
  return key;
};

export const undo = (key: string) => {
  taken(key);
};

const subscribe = (change: () => void) => {
  told.add(change);
  return () => {
    told.delete(change);
  };
};

export const useGoing = (): readonly Going[] => useSyncExternalStore(subscribe, () => waiting);

const NONE: ReadonlySet<string> = new Set();
let hiddenOf: { from: readonly Going[]; ids: ReadonlySet<string> } = { from: [], ids: NONE };

export const useHiddenGoing = (): ReadonlySet<string> => {
  const now = useGoing();
  if (hiddenOf.from !== now)
    hiddenOf = { from: now, ids: new Set(now.flatMap((one) => one.hides)) };
  return hiddenOf.ids;
};
