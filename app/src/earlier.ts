import { invoke } from "@tauri-apps/api/core";
import type { Doc } from "./core";
import type { Signing } from "./crediting";
import { stamped, weigh } from "./format";
import { fill, t } from "./locales";
import { agentNamed, clientNamed, machineOf } from "./who";

export interface Earlier {
  print: string;
  wrote?: string;
  replaced: string;
  via?: string;
  by?: string;
  keptBy?: string;
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

export type Compared =
  | { kind: "drawn"; html: string; gone: number; came: number }
  | { kind: "tooLong" };

/** Without `now`, `was` is compared with what the document says on disk. */
export const docCompared = (id: string, was: string, now: string | null): Promise<Compared> =>
  invoke("doc_compared", { id, was, now });

export const whenOf = (one: Earlier): string => stamped(one.wrote ?? one.replaced);

// A version the log never named has nobody to say wrote it, which is not the same as the person.
export const writerOf = (one: Earlier, signing: Signing = "named"): string | undefined => {
  if (!one.by) return undefined;
  if (one.via) return signing === "named" ? clientNamed(one.via) : t("anAgent");
  return agentNamed(one.by) ? t("anAssistant") : t("earlierByYou");
};

export const awayOf = (one: Earlier): string => {
  const machine = machineOf(one.keptBy);
  return machine ? fill("earlierAwayOn", machine) : t("earlierAway");
};

export const saidOf = (one: Earlier, signing?: Signing): string =>
  [whenOf(one), writerOf(one, signing), weigh(one.bytes)].filter(Boolean).join(" · ");
