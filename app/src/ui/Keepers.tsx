import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { chooseSync, type Keeper, keeperOf, keepers, makeRoom, type Offering } from "../core";
import { warningOf } from "../keepers";
import { t } from "../locales";
import { saidPlainly } from "../refusal";

interface Props {
  busy: boolean;
  onTrouble: (said: string) => void;
  onDeciding?: (deciding: boolean) => void;
  onDone: (at?: string) => void;
}

interface Standing {
  at: string;
  keeper: Keeper;
  named?: string;
}

const HUE: Record<string, string> = {
  drive: "bg-[#1a73e8]",
  onedrive: "bg-[#0f6cbd]",
  icloud: "bg-[#5b8def]",
  dropbox: "bg-[#0061ff]",
};

const row =
  "flex w-full items-center gap-2.5 rounded-lg border px-3 py-2 text-left hover:bg-hover disabled:opacity-50 disabled:hover:bg-transparent";
const badge = "flex-none rounded-md px-1.5 py-0.5 text-[10px] tracking-wide";

export default function Keepers({ busy, onTrouble, onDeciding, onDone }: Props) {
  const [offers, setOffers] = useState<Offering[]>([]);
  const [standing, setStanding] = useState<Standing>();
  const [held, setHeld] = useState(false);

  useEffect(() => {
    keepers()
      .then((found) => setOffers(found ?? []))
      .catch((e) => onTrouble(saidPlainly(e)));
  }, [onTrouble]);

  useEffect(() => {
    onDeciding?.(Boolean(standing));
  }, [standing, onDeciding]);

  const stuck = busy || held;

  const browse = () => {
    if (stuck) return;
    setHeld(true);
    open({ directory: true })
      .then(async (at) => {
        if (typeof at !== "string") return;
        const told = await keeperOf(at);
        setStanding({ at, keeper: told.keeper, named: told.named });
      })
      .catch((e) => onTrouble(saidPlainly(e)))
      .finally(() => setHeld(false));
  };

  const take = (one: Offering) => {
    if (stuck || !one.at || !one.into) return;
    setStanding({ at: one.into, keeper: "cloud", named: one.named });
  };

  const settle = (work: Promise<unknown>, at?: string) => {
    setHeld(true);
    work
      .then(() => onDone(at))
      .catch((e) => {
        onTrouble(saidPlainly(e));
        setHeld(false);
      });
  };

  const keep = () => {
    if (stuck || !standing) return;
    settle(
      makeRoom(standing.at).then(() => chooseSync(standing.at)),
      standing.at,
    );
  };

  const alone = () => {
    if (stuck) return;
    settle(chooseSync(undefined));
  };

  if (standing) {
    const warning = warningOf(standing.keeper, standing.named);
    return (
      <div className="flex flex-col gap-3">
        <div className="flex items-center gap-2.5 rounded-lg border border-accent bg-accent-soft px-3 py-2">
          <span className="min-w-0 flex-1">
            <span className="block text-[13.5px] font-medium">
              {standing.named ?? t("keepersOther")}
            </span>
            <span className="block truncate text-[11px] text-faint">{standing.at}</span>
          </span>
        </div>

        <div
          className={`rounded-lg border px-3 py-2 text-xs leading-relaxed ${
            warning.mild ? "border-hair bg-accent-soft text-soft" : "border-hue-amber/40 text-soft"
          }`}
        >
          <span className="block text-[12.5px] font-semibold text-ink">{warning.said}</span>
          {warning.why}
        </div>

        <div className="flex items-center gap-4 text-xs">
          <button
            type="button"
            disabled={stuck}
            onClick={() => setStanding(undefined)}
            className="text-faint hover:text-ink disabled:opacity-60"
          >
            {t("welcomeBack")}
          </button>
          <button
            type="button"
            disabled={stuck}
            onClick={keep}
            className="ml-auto rounded-lg bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60"
          >
            {t("keepersSave")}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-2">
      {offers.map((one) => (
        <button
          key={one.key}
          type="button"
          disabled={stuck || !one.at}
          onClick={() => take(one)}
          className={`${row} border-line`}
        >
          <span
            className={`grid size-6 flex-none place-items-center rounded-md text-[11px] font-semibold text-white ${
              HUE[one.key] ?? "bg-soft"
            }`}
          >
            {one.named.slice(0, 1)}
          </span>
          <span className="min-w-0 flex-1">
            <span className="block text-[13.5px] font-medium">{one.named}</span>
            <span className="block truncate text-[11px] text-faint">
              {one.at ?? t("keepersMissing")}
            </span>
          </span>
          <span
            className={`${badge} ${
              one.at ? "bg-hue-green/15 text-hue-green" : "border border-hair text-faint"
            }`}
          >
            {one.at ? t("keepersHere") : t("keepersGone")}
          </span>
        </button>
      ))}

      <button type="button" disabled={stuck} onClick={browse} className={`${row} border-line`}>
        <span className="grid size-6 flex-none place-items-center rounded-md border border-dashed border-line text-[11px] text-faint">
          +
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-[13.5px] font-medium">{t("keepersOther")}</span>
          <span className="block truncate text-[11px] text-faint">{t("keepersOtherWhy")}</span>
        </span>
      </button>

      <div className="mt-1 border-t border-hair pt-3">
        <button type="button" disabled={stuck} onClick={alone} className={`${row} border-line`}>
          <span className="grid size-6 flex-none place-items-center rounded-md bg-soft text-[11px] font-semibold text-bg">
            1
          </span>
          <span className="min-w-0 flex-1">
            <span className="block text-[13.5px] font-medium">{t("welcomeAlone")}</span>
            <span className="block truncate text-[11px] text-faint">{t("welcomeAloneWhy")}</span>
          </span>
        </button>
      </div>
    </div>
  );
}
