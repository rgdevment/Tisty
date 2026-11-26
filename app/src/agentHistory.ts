import { invoke } from "@tauri-apps/api/core";
import type { Earlier, Shelf } from "./earlier";

/** A copy of an agent's file Tisty kept on this computer before writing over it; its words are read apart. */
export interface Copy {
  readonly at: string;
  readonly named: string;
  readonly n: number;
  readonly bytes: number;
  readonly pinned: boolean;
  readonly when?: string;
}

export interface AgentHistory extends Shelf {
  copies: Copy[];
}

export type Row = { kind: "version"; one: Earlier } | { kind: "copy"; one: Copy };

export const agentHistory = (file: string): Promise<AgentHistory> =>
  invoke("agent_history", { file });

export const copyKey = (one: Copy): string => `copy:${one.n}:${one.at}`;

export const keyOf = (row: Row): string =>
  row.kind === "version" ? row.one.print : copyKey(row.one);

const timeOf = (one: Earlier): number => Date.parse(one.wrote ?? one.replaced);

// A copy's number orders it among its file's copies, and its date only places it among the versions.
const placed = (copies: readonly Copy[]): { one: Copy; at: number }[] => {
  const files = [...new Set(copies.map((one) => one.at))];
  return files
    .flatMap((file) => {
      let last = Number.POSITIVE_INFINITY;
      return copies
        .filter((one) => one.at === file)
        .sort((one, two) => two.n - one.n)
        .map((one) => {
          const at = one.when ? Date.parse(one.when) : Number.NaN;
          last = Number.isNaN(at) ? last : Math.min(at, last);
          return { one, at: last };
        });
    })
    .sort((one, two) => Number(two.at > one.at) - Number(two.at < one.at));
};

/** Newest first, versions in the order they come, and each file's original fixed at the end. */
export const timeline = (versions: readonly Earlier[], copies: readonly Copy[]): Row[] => {
  const waiting = placed(copies.filter((one) => !one.pinned));
  const rows: Row[] = [];
  for (const one of versions) {
    const at = timeOf(one);
    while (waiting.length > 0 && waiting[0].at >= at) {
      rows.push({ kind: "copy", one: waiting[0].one });
      waiting.shift();
    }
    rows.push({ kind: "version", one });
  }
  rows.push(...waiting.map(({ one }): Row => ({ kind: "copy", one })));
  rows.push(...copies.filter((one) => one.pinned).map((one): Row => ({ kind: "copy", one })));
  return rows;
};
