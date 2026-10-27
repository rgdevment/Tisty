import { choiceWaits } from "./coming";
import type { Carrying, CloudStatus } from "./core";

export type Footing =
  | { kind: "ok" }
  | { kind: "syncing"; done: number; whole: number }
  | { kind: "wait"; troubled: boolean }
  | { kind: "halted"; until: number }
  | { kind: "spent" }
  | { kind: "offline" }
  | { kind: "lost" }
  | { kind: "paused" }
  | { kind: "look"; putOff: boolean }
  | { kind: "first" }
  | { kind: "sealing"; turn: "on" | "off" };

export interface Seen {
  state: Carrying | null;
  status: CloudStatus | null;
  coming: { done: number; whole: number } | null;
  mine: boolean;
  offline: boolean;
  knocked: boolean;
  putOff: number;
  stuck: unknown;
  now?: number;
}

type Said = "lost" | "wait" | "troubled" | "offline" | "spent" | "look";

const SAID: Record<string, Said> = {
  cloudLost: "lost",
  cloudElsewhere: "lost",
  cloudNotNow: "wait",
  cloudSealing: "wait",
  "deferred-limit": "wait",
  "deferred-troubled": "troubled",
  "deferred-unsent": "troubled",
  "deferred-offline": "offline",
  "deferred-budget": "spent",
  cloudSpent: "spent",
};

// Busy is another round of this computer's holding the cloud, which passes by itself.
const QUIET = new Set([
  "",
  "ok",
  "deferred-changed",
  "deferred-paused",
  "deferred-busy",
  "cloudSealingHere",
]);

const DAY = 86_400;

const codeOf = (said: unknown): string => (said as { code?: string } | null)?.code ?? "";

const saidOf = (code: string, at: number, now: number): Said | null => {
  if (QUIET.has(code)) return null;
  const said = SAID[code] ?? "look";
  // A day's limit ends when the day does, whatever the last round said.
  if (said === "spent" && Math.floor(at / DAY) < Math.floor(now / DAY)) return null;
  return said;
};

export const synced = (state: Carrying | null): boolean =>
  state?.mode === "cloud" || (state?.mode === "folder" && Boolean(state.chosen));

// What needs the person outranks what passes by itself, so a wait never hides a choice owed.
export const footing = (seen: Seen): Footing | null => {
  const { state, status } = seen;
  if (!synced(state)) return null;
  const now = seen.now ?? Date.now() / 1000;
  if (seen.coming || seen.mine || status?.running)
    return { kind: "syncing", done: seen.coming?.done ?? 0, whole: seen.coming?.whole ?? 0 };
  const said = new Set<Said | null>([
    status?.last ? saidOf(status.last.code, status.last.at, now) : null,
    saidOf(codeOf(seen.stuck), now, now),
  ]);
  if (said.has("lost")) return { kind: "lost" };
  if (status?.pausedHere) return { kind: "paused" };
  if (seen.putOff > 0) return { kind: "look", putOff: true };
  if (seen.knocked || choiceWaits(seen.stuck) || status?.lost || said.has("look"))
    return { kind: "look", putOff: false };
  if (status?.haltedUntil && status.haltedUntil > now)
    return { kind: "halted", until: status.haltedUntil };
  if (said.has("spent")) return { kind: "spent" };
  if (state?.mode === "cloud" && (seen.offline || said.has("offline"))) return { kind: "offline" };
  if (said.has("wait") || said.has("troubled"))
    return { kind: "wait", troubled: !said.has("wait") };
  if (status?.sealingElsewhere) return { kind: "sealing", turn: status.sealingElsewhere };
  if (state?.mode === "cloud" && status && !status.joined) return { kind: "first" };
  return { kind: "ok" };
};

export const offered = (foot: Footing): boolean =>
  foot.kind === "ok" ||
  foot.kind === "first" ||
  foot.kind === "paused" ||
  foot.kind === "lost" ||
  foot.kind === "look";
