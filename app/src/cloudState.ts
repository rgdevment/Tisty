import type { CloudStatus } from "./core";
import type { t } from "./locales";

type Key = Parameters<typeof t>[0];

export type Exit = "reconnect" | "storage" | "turnOn";

export interface Standing {
  said: Key;
  why: Key;
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

// The switch and a pause for every machine outrank whatever the last round said, which may predate them.
export const cloudStanding = (status: CloudStatus, now = Date.now() / 1000): Standing | null => {
  if (status.pausedHere)
    return { said: "stateOff", why: "stateOffWhy", exit: "turnOn", exitSaid: "cloudTurnOn" };
  if (status.haltedUntil && status.haltedUntil > now)
    return { said: "stateHalted", why: "stateHaltedWhy" };
  const code = status.last?.code;
  const found = code ? BY_CODE[code] : undefined;
  return found ? { ...found, name: status.last?.name ?? undefined } : null;
};
