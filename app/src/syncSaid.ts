import type { Carried, Settled } from "./core";
import { stamped } from "./format";
import { fill, t } from "./locales";

type Word = Parameters<typeof t>[0];

const carried: Record<Exclude<Carried, "busy">, Word> = {
  came: "syncCame",
  sent: "syncSent",
  both: "syncBoth",
  same: "syncSame",
};

const counted = (many: number, one: Word, more: Word): string | null =>
  many === 0 ? null : many === 1 ? t(one) : fill(more, String(many));

export const syncSaid = (said: Settled, at = new Date()): string => {
  if (said.carried === "busy") return t("syncBusy");
  const parts = [
    counted(said.came ?? 0, "syncResultCameOne", "syncResultCame"),
    counted(said.went ?? 0, "syncResultWentOne", "syncResultWent"),
    counted(said.unconfirmed?.length ?? 0, "syncResultMachineWaits", "syncResultMachinesWait"),
    counted(said.waiting?.length ?? 0, "syncResultDocWaits", "syncResultDocsWait"),
  ].filter((one): one is string => one !== null);
  return fill("syncResultAt", stamped(at.toISOString(), at)).concat(
    ": ",
    parts.length ? parts.join(", ") : t(carried[said.carried]),
  );
};
