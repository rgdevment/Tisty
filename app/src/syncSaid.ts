import type { Carried, Cause, Settled } from "./core";
import { budgetResets, stamped } from "./format";
import { fill, t, type Word } from "./locales";
import { named } from "./previews";

const carried: Record<Carried, Parameters<typeof t>[0]> = {
  came: "syncCame",
  sent: "syncSent",
  both: "syncBoth",
  same: "syncSame",
  busy: "syncBusy",
};

const TAKING_LONG = 60;

/** A round that has run past a minute says for how long, so it never looks frozen. */
export const syncingSaid = (secs: number): string =>
  secs >= TAKING_LONG ? fill("syncingFor", String(Math.floor(secs / 60))) : t("syncing_");

export const syncSaid = (said: Settled, at = new Date()): string =>
  said.carried === "busy"
    ? (said.running ?? 0) >= TAKING_LONG
      ? syncingSaid(said.running ?? 0)
      : t("syncBusy")
    : `${fill("syncResultAt", stamped(at.toISOString(), at))}: ${t(carried[said.carried])}`;

const CAUSES: Record<Cause, Word> = {
  blocked: "roundBlocked",
  filtered: "roundFiltered",
  slow: "roundSlow",
};

const LASTING_ROUNDS = 2;
const LASTING_SECS = 120;

export const offlineNow = () => typeof navigator !== "undefined" && navigator.onLine === false;

/** What stood in the way of the cloud, unless this computer is simply offline, which says so itself. */
export const causeSaid = (cause?: Cause | null): string | null =>
  cause && cause in CAUSES && !offlineNow() ? fill(CAUSES[cause], t("cloudGoogle")) : null;

interface Failing {
  cause?: Cause | null;
  rounds?: number;
  since?: number;
  at?: number;
}

// One failed round is a blip; only the same cause twice in a row, or for two minutes, is said.
export const lastingCause = (said: Failing, now = Date.now() / 1000): string | null => {
  const rounds = said.rounds ?? 0;
  const lasted = said.since ? (said.at ?? now) - said.since : 0;
  return rounds >= LASTING_ROUNDS || lasted >= LASTING_SECS ? causeSaid(said.cause) : null;
};

/** Why a round waits, with a spent day told by the hour it turns here. */
export const waitSaid = (key: Parameters<typeof t>[0], now = new Date()): string =>
  key === "deferredBudget" ? fill(key, budgetResets(now)) : t(key);

const NAMED = 3;

// The attachments a round left for later, by the name the person gave them; the rest only counted.
export const whatsLeft = (said: Settled): string | null => {
  const waiting = said.deferred;
  if (!waiting || waiting.left === 0) return null;
  const shown = waiting.names.slice(0, NAMED).map(named);
  if (!shown.length) return fill("deferredCount", String(waiting.left));
  const more = waiting.left - shown.length;
  const list = more > 0 ? fill("deferredMore", shown.join(", "), String(more)) : shown.join(", ");
  return fill("deferredLeft", String(waiting.left), list);
};
