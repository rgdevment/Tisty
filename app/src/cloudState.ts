import { type CloudStatus, cloudForget, cloudStatus } from "./core";
import type { t } from "./locales";
import { plainly } from "./refusal";

type Key = Parameters<typeof t>[0];

export type Exit = "reconnect" | "storage" | "turnOn";

export interface Standing {
  said: Key;
  why?: Key;
  told?: string;
  exit?: Exit;
  exitSaid?: Key;
  name?: string;
}

export const STORAGE = "https://drive.google.com/settings/storage";

const BY_CODE: Record<string, Standing> = {
  cloudLost: {
    said: "stateReconnect",
    why: "stateReconnectWhy",
    exit: "reconnect",
    exitSaid: "stateReconnectDo",
  },
  cloudElsewhere: {
    said: "stateElsewhere",
    why: "stateElsewhereWhy",
    exit: "reconnect",
    exitSaid: "stateElsewhereDo",
  },
  cloudFull: { said: "stateFull", why: "stateFullWhy", exit: "storage", exitSaid: "stateFullDo" },
  "deferred-offline": { said: "stateOffline", why: "stateOfflineWhy" },
  "deferred-budget": { said: "stateBudget", why: "stateBudgetWhy" },
  "deferred-limit": { said: "stateWait", why: "stateWaitWhy" },
  "deferred-busy": { said: "stateWait", why: "stateWaitWhy" },
  cloudNotNow: { said: "stateWait", why: "stateWaitWhy" },
};

const QUIET = new Set(["ok", "deferred-changed", "deferred-paused"]);

const DAY = 86_400;

// The switch and a pause for every machine outrank whatever the last round said, which may predate them.
export const cloudStanding = (status: CloudStatus, now = Date.now() / 1000): Standing | null => {
  if (status.pausedHere)
    return { said: "stateOff", why: "stateOffWhy", exit: "turnOn", exitSaid: "cloudTurnOn" };
  if (status.haltedUntil && status.haltedUntil > now)
    return { said: "stateHalted", why: "stateHaltedWhy" };
  const last = status.last;
  if (!last || QUIET.has(last.code)) return null;
  if (last.code === "deferred-budget" && Math.floor(last.at / DAY) < Math.floor(now / DAY))
    return null;
  const name = last.name ?? undefined;
  const found = BY_CODE[last.code];
  if (found) return { ...found, name };
  return { said: "stateFailed", told: plainly({ code: last.code, name }) };
};

// An account connected and then left is let go of, unless it is the one this copy syncs with by now.
export const letGoUnused = (account: string): void => {
  cloudStatus()
    .then((now) => (now?.account === account ? undefined : cloudForget(account)))
    .catch(() => undefined);
};
