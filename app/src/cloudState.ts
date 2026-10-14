import { apartWaits } from "./coming";
import { type CloudStatus, cloudForget, cloudStatus } from "./core";
import { budgetResets } from "./format";
import { fill, t } from "./locales";
import { plainly } from "./refusal";
import { lastingCause } from "./syncSaid";

type Key = Parameters<typeof t>[0];

export type Exit = "reconnect" | "storage" | "turnOn" | "again" | "choose";

export interface Standing {
  said: Key;
  why?: Key;
  told?: string;
  exit?: Exit;
  exitSaid?: Key;
  name?: string;
  /** Said in red: what keeps standing in the way, not a wait that ends by itself. */
  urgent?: boolean;
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
  cloudEmptied: {
    said: "stateEmptied",
    why: "stateEmptiedWhy",
    exit: "again",
    exitSaid: "stateEmptiedDo",
  },
  cloudFull: { said: "stateFull", why: "stateFullWhy", exit: "storage", exitSaid: "stateFullDo" },
  "deferred-offline": { said: "stateOffline", why: "stateOfflineWhy" },
  "deferred-limit": { said: "stateWait", why: "stateWaitWhy" },
  "deferred-troubled": { said: "stateTroubled", why: "stateTroubledWhy" },
  "deferred-unsent": { said: "stateUnsent", why: "stateUnsentWhy" },
  "deferred-busy": { said: "stateWait", why: "stateWaitWhy" },
  cloudNotNow: { said: "stateWait", why: "stateWaitWhy" },
};

const APART: Standing = {
  said: "stateApart",
  why: "stateApartWhy",
  exit: "choose",
  exitSaid: "stateApartDo",
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
  const spent = last.code === "deferred-budget" || last.code === "cloudSpent";
  if (spent && Math.floor(last.at / DAY) < Math.floor(now / DAY)) return null;
  if (spent)
    return {
      said: "stateBudget",
      told: fill("stateBudgetWhy", budgetResets(new Date(now * 1000))),
    };
  if (apartWaits(last)) return status.adopts ? null : { ...APART, name: t("cloudGoogle") };
  if (last.code === "restoredApart")
    return {
      said: "stateRestoredApart",
      told: fill("stateRestoredApartWhy", t("cloudGoogle")),
      exit: "choose",
      exitSaid: "stateApartDo",
    };
  const cause = last.code === "deferred-offline" ? lastingCause(last, now) : null;
  if (cause) return { said: "stateOffline", told: cause, urgent: true };
  const name = last.name ?? undefined;
  const found = BY_CODE[last.code];
  if (found) return { ...found, name };
  return { said: "stateFailed", told: plainly({ code: last.code, name }) };
};

// An account connected and then left is let go of, unless it is the one this copy syncs with by now.
export const letGoUnused = ({
  account,
  provider,
}: {
  account: string;
  provider?: string;
}): void => {
  cloudStatus()
    .then((now) =>
      now?.account === account && (!provider || now.provider === provider)
        ? undefined
        : cloudForget(account, provider),
    )
    .catch(() => undefined);
};
