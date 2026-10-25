import { CLOUDS, cloudOf } from "./clouds";
import { apartWaits } from "./coming";
import { type CloudStatus, cloudForget, cloudStatus } from "./core";
import { budgetResets } from "./format";
import { cloudNamed, fill, t } from "./locales";
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

export const followedTo = (stuck: unknown): string | undefined => {
  const told = stuck as { code?: string; name?: string } | null;
  if (told?.code !== "movedToCloud" || !told.name) return undefined;
  return CLOUDS.find((one) => told.name?.startsWith(`${cloudNamed(one.id)} · `))?.id;
};

export const storageOf = (provider?: string | null): string | undefined =>
  cloudOf(provider ?? "google")?.storage;

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
  cloudNotNow: { said: "stateWait", why: "stateWaitWhy" },
  cloudSealed: { said: "stateSealed", why: "stateSealedWhy" },
  cloudSealing: { said: "stateSealing", why: "stateSealingWhy" },
  cloudOtherPhrase: { said: "stateOtherPhrase", why: "stateOtherPhraseWhy", urgent: true },
  cloudUnsealed: { said: "stateUnsealed", why: "stateUnsealedWhy", urgent: true },
  cloudMarkDamaged: { said: "stateMarkDamaged", why: "stateMarkDamagedWhy", urgent: true },
};

const APART: Standing = {
  said: "stateApart",
  why: "stateApartWhy",
  exit: "choose",
  exitSaid: "stateApartDo",
};

// Busy is another round of this computer's holding the cloud, not the cloud asking to wait.
const QUIET = new Set(["ok", "deferred-changed", "deferred-paused", "deferred-busy"]);

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
  if (apartWaits(last)) return status.adopts ? null : { ...APART, name: t("cloudChosen") };
  if (last.code === "restoredApart")
    return {
      said: "stateRestoredApart",
      told: fill("stateRestoredApartWhy", t("cloudChosen")),
      exit: "choose",
      exitSaid: "stateApartDo",
    };
  const cause = last.code === "deferred-offline" ? lastingCause(last, now) : null;
  if (cause) return { said: "stateOffline", told: cause, urgent: true };
  const name = last.name ?? undefined;
  const found = BY_CODE[last.code];
  // A cloud with no storage page of its own, like a WebDAV server, is told its state with no button that goes nowhere.
  if (found?.exit === "storage" && !storageOf(status.provider))
    return { said: found.said, why: found.why, name };
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
