import { listen } from "@tauri-apps/api/event";
import { useSyncExternalStore } from "react";
import { cloudWipe } from "./core";

export interface Erased {
  gone: number;
  of: number;
}

export interface Erasing {
  account: string;
  going: boolean;
  erased?: Erased;
  gone?: number;
  failed?: unknown;
}

let now: Erasing | null = null;
const told = new Set<() => void>();

const set = (next: Erasing | null) => {
  now = next;
  for (const one of told) one();
};

// Kept outside the dialog, so closing it leaves the erasing going and opening it again shows where it stands.
export const erase = async (account: string, email: string, provider?: string) => {
  if (now?.going) return;
  set({ account, going: true });
  // Listened to before asking, so the first counts are not said to nobody.
  const off = await listen<Erased>("wiping", (step) => {
    if (now?.going) set({ ...now, erased: step.payload });
  }).catch(() => undefined);
  await cloudWipe(account, email, provider)
    .then((gone) => set({ account, going: false, gone }))
    .catch((failed) => set({ account, going: false, failed }));
  off?.();
};

export const forgetErasing = () => {
  if (!now?.going) set(null);
};

const watch = (then: () => void) => {
  told.add(then);
  return () => {
    told.delete(then);
  };
};

export const useErasing = () => useSyncExternalStore(watch, () => now);
