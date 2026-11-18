import { invoke } from "@tauri-apps/api/core";
import type { Doc } from "./core";
import { stamped, weigh } from "./format";
import { fill, t } from "./locales";
import { agentNamed, clientNamed, machineOf } from "./who";

export interface Earlier {
  print: string;
  wrote?: string;
  replaced: string;
  via?: string;
  by?: string;
  bytes: number;
  here: boolean;
}

/** `kept` counts every body the shelf holds, the ones the list leaves out too. */
export interface Shelf {
  versions: Earlier[];
  kept: number;
}

export const HISTORY_DAYS = [1, 5, 15, 30] as const;

export const docVersions = (id: string): Promise<Shelf> => invoke("doc_versions", { id });
export const docVersion = (id: string, print: string): Promise<string> =>
  invoke("doc_version", { id, print });
export const docBackTo = (id: string, print: string): Promise<Doc> =>
  invoke("doc_back_to", { id, print });
export const docHistoryEmpty = (id: string): Promise<number> => invoke("doc_history_empty", { id });
export const keepHistoryDays = (days: number): Promise<number> =>
  invoke("keep_history_days", { days });

export const whenOf = (one: Earlier): string => stamped(one.wrote ?? one.replaced);

// A version the log never named has nobody to say wrote it, which is not the same as the person.
export const writerOf = (one: Earlier): string | undefined => {
  if (!one.by) return undefined;
  if (one.via) return clientNamed(one.via);
  return agentNamed(one.by) ? t("anAssistant") : t("earlierByYou");
};

export const awayOf = (one: Earlier): string => {
  const machine = machineOf(one.by);
  return machine ? fill("earlierAwayOn", machine) : t("earlierAway");
};

export const saidOf = (one: Earlier): string =>
  [whenOf(one), writerOf(one), weigh(one.bytes)].filter(Boolean).join(" · ");
