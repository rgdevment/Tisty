import { useSyncExternalStore } from "react";

export type Stored<T> = {
  current: () => T;
  put: (next: T) => void;
  use: () => T;
};

export const stored = <T>(key: string, read: (said: Record<string, unknown>) => T): Stored<T> => {
  const told = new Set<() => void>();
  let now: { value: T } | null = null;

  const kept = (): T => {
    try {
      const said: unknown = JSON.parse(localStorage.getItem(key) ?? "{}");
      return read(said && typeof said === "object" ? (said as Record<string, unknown>) : {});
    } catch {
      return read({});
    }
  };

  const current = (): T => {
    now ??= { value: kept() };
    return now.value;
  };

  const put = (next: T) => {
    now = { value: next };
    try {
      localStorage.setItem(key, JSON.stringify(next));
    } catch {}
    for (const one of told) one();
  };

  const subscribe = (change: () => void) => {
    const elsewhere = (seen: StorageEvent) => {
      if (seen.key !== key) return;
      now = null;
      change();
    };
    told.add(change);
    window.addEventListener("storage", elsewhere);
    return () => {
      told.delete(change);
      window.removeEventListener("storage", elsewhere);
    };
  };

  return { current, put, use: () => useSyncExternalStore(subscribe, current) };
};
