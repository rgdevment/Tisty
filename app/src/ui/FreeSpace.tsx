import { useEffect, useState } from "react";
import {
  type CloudStatus,
  cloudStatus,
  type Freeing,
  freeSpace,
  stopFreeing,
  type Unused,
  unusedAttachments,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Card, { type Which, type Word } from "./Card";
import { mild, off } from "./Rows";

const DAYS = [30, 60, 90];

interface Props {
  busy: Which | null;
  freeing: Freeing | null;
  setFreeing: (freeing: Freeing | null) => void;
  days: number;
  setDays: (days: number) => void;
  said?: Word;
  trouble?: Word;
  fail: (word?: Word) => void;
}

/** Only with a cloud reached through its API, where every attachment stays here once opened. */
export default function FreeSpace({
  busy,
  freeing,
  setFreeing,
  days,
  setDays,
  said,
  trouble,
  fail,
}: Props) {
  const [found, setFound] = useState<Unused | null>(null);
  const [spent, setSpent] = useState<CloudStatus | null>(null);
  const held = busy !== null;
  const running = freeing !== null && !freeing.done;

  useEffect(() => {
    if (running) return;
    let live = true;
    setFound(null);
    unusedAttachments(days)
      .then((one) => live && setFound(one))
      .catch(() => live && setFound(null));
    return () => {
      live = false;
    };
  }, [days, running]);

  useEffect(() => {
    if (running) return;
    cloudStatus()
      .then(setSpent)
      .catch(() => setSpent(null));
  }, [running]);

  const letGo = () => {
    fail(undefined);
    setFreeing({ gone: 0, freed: 0, done: false });
    freeSpace(days).catch((e) => {
      setFreeing(null);
      fail({ card: "free", text: saidPlainly(e) });
    });
  };

  return (
    <Card title={t("freeTitle")} which="free" busy={busy} said={said} trouble={trouble}>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("freeWhy")}</p>
      <p className="mt-1 text-[12.5px] leading-relaxed text-soft">{t("freeQuota")}</p>
      {spent && spent.unitsADay > 0 && (
        <p className="mt-1 text-[12.5px] leading-relaxed text-faint tabular-nums">
          {t("cloudToday")}:{" "}
          {fill(
            "cloudTodayUnits",
            spent.unitsSpent.toLocaleString(),
            spent.unitsADay.toLocaleString(),
          )}
          {spent.bytesADay > 0 &&
            ` · ${fill("cloudTodayBytes", weigh(spent.bytesSpent), weigh(spent.bytesADay))}`}
        </p>
      )}
      {found && (
        <p className="mt-1 text-[12.5px] leading-relaxed text-soft">
          {found.files > 0
            ? fill("freeFound", String(found.files), weigh(found.bytes))
            : t("freeNone")}
        </p>
      )}
      <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
        <select
          aria-label={t("freeUnused")}
          value={String(days)}
          disabled={held || running}
          onChange={(e) => setDays(Number(e.target.value))}
          className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
        >
          {DAYS.map((one) => (
            <option key={one} value={one}>
              {fill("freeDays", String(one))}
            </option>
          ))}
        </select>
        <button
          type="button"
          disabled={held || running || !found || found.files === 0}
          onClick={letGo}
          className={mild}
        >
          {t("freeDo")}
        </button>
      </div>
      {freeing && (
        <div className="mt-2 flex items-center gap-2.5">
          <span className="text-[11.5px] leading-relaxed text-soft">
            {fill(freeing.done ? "freed" : "freeing", weigh(freeing.freed))}
          </span>
          {running && (
            <button
              type="button"
              onClick={() => void stopFreeing()}
              className="rounded-md border border-line px-2.5 py-0.5 text-[11.5px] text-soft hover:border-urgent hover:text-urgent"
            >
              {t("holdsStop")}
            </button>
          )}
        </div>
      )}
    </Card>
  );
}
