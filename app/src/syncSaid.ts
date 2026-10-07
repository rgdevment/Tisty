import type { Carried, Settled } from "./core";
import { stamped } from "./format";
import { fill, t } from "./locales";

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
