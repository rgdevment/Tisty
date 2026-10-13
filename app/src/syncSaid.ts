import type { Carried, Settled } from "./core";
import { budgetResets, stamped } from "./format";
import { fill, t } from "./locales";
import { named } from "./previews";

const carried: Record<Carried, Parameters<typeof t>[0]> = {
  came: "syncCame",
  sent: "syncSent",
  both: "syncBoth",
  same: "syncSame",
  busy: "syncBusy",
};

export const syncSaid = (said: Settled, at = new Date()): string =>
  said.carried === "busy"
    ? t("syncBusy")
    : `${fill("syncResultAt", stamped(at.toISOString(), at))}: ${t(carried[said.carried])}`;

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
