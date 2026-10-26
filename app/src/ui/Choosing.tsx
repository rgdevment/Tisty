import { useEffect, useState } from "react";
import { chooseSync } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Clouding from "./Clouding";
import Keepers from "./Keepers";

interface Props {
  busy: boolean;
  offered: boolean;
  start?: Way;
  onTrouble: (said?: string) => void;
  onDeciding?: (deciding: boolean) => void;
  onDone: (at?: string) => void;
}

export type Way = "pick" | "cloud" | "folder";

const row =
  "flex w-full items-center gap-2.5 rounded-[10px] border border-line px-3 py-2 text-left hover:bg-hover disabled:opacity-50 disabled:hover:bg-transparent";

function Option({
  mark,
  said,
  why,
  busy,
  onPick,
}: {
  mark: string;
  said: string;
  why: string;
  busy: boolean;
  onPick: () => void;
}) {
  return (
    <button type="button" disabled={busy} onClick={onPick} className={row}>
      <span className="grid size-6 flex-none place-items-center rounded-md border border-line text-[11.5px] text-soft">
        {mark}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-[13px] font-medium">{said}</span>
        <span className="block text-[11.5px] text-faint">{why}</span>
      </span>
    </button>
  );
}

export default function Choosing({
  busy,
  offered,
  start = "pick",
  onTrouble,
  onDeciding,
  onDone,
}: Props) {
  const [way, setWay] = useState<Way>(start);
  const [held, setHeld] = useState(false);
  const [standing, setStanding] = useState(false);

  useEffect(() => {
    onDeciding?.(way !== "pick");
  }, [way, onDeciding]);

  const stuck = busy || held;

  const alone = () => {
    if (stuck) return;
    setHeld(true);
    onTrouble(undefined);
    chooseSync(undefined)
      .then(() => onDone(undefined))
      .catch((e) => {
        onTrouble(saidPlainly(e));
        setHeld(false);
      });
  };

  const back = () => {
    onTrouble(undefined);
    setWay("pick");
  };

  if (way === "cloud")
    return <Clouding busy={busy} onTrouble={onTrouble} onBack={back} onDone={onDone} />;

  if (way === "folder")
    return (
      <div className="flex flex-col gap-3">
        <p className="text-[12.5px] leading-relaxed text-soft">{t("keepersWhy")}</p>
        <Keepers
          busy={busy}
          onTrouble={(said) => onTrouble(said)}
          onDeciding={setStanding}
          onDone={onDone}
        />
        {!standing && (
          <button
            type="button"
            disabled={busy}
            onClick={back}
            className="self-start text-[11.5px] text-faint hover:text-ink disabled:opacity-60"
          >
            {t("welcomeBack")}
          </button>
        )}
      </div>
    );

  return (
    <div className="flex flex-col gap-2">
      <Option
        mark="▣"
        said={t("welcomeAlone")}
        why={t("welcomeAloneWhy")}
        busy={stuck}
        onPick={alone}
      />
      {offered && (
        <Option
          mark="☁"
          said={t("chooseCloud")}
          why={t("chooseCloudWhy")}
          busy={stuck}
          onPick={() => setWay("cloud")}
        />
      )}
      <Option
        mark="▤"
        said={t("chooseFolder")}
        why={t("chooseFolderWhy")}
        busy={stuck}
        onPick={() => setWay("folder")}
      />
      {held && (
        <p role="status" className="text-[11.5px] text-faint motion-safe:animate-pulse">
          {t("movingStarting")}
        </p>
      )}
      <p className="mt-1 text-[11.5px] leading-relaxed text-faint">{t("chooseOneAtATime")}</p>
    </div>
  );
}
