import type { Settings } from "../core";
import { HISTORY_DAYS, keepHistoryDays } from "../earlier";
import { fill, t } from "../locales";
import type { Run, Word } from "./Card";
import { Band, Line, off } from "./Rows";
import { More, Scope } from "./Zone";

interface Props {
  kept: Settings;
  amend: (edit: (was: Settings) => Settings) => void;
  held: boolean;
  run: Run;
  tell: (word?: Word) => void;
}

export default function EarlierLimit({ kept, amend, held, run, tell }: Props) {
  return (
    <>
      <Band label={t("earlierBand")} />
      <div className="border-t border-hair">
        <Line
          row="earlier"
          title={
            <span className="flex flex-wrap items-center gap-1.5">
              {t("earlierLimit")}
              <Scope here />
            </span>
          }
          which="earlier"
          more={<More>{t("earlierLimitWhy")}</More>}
        >
          <select
            aria-label={t("earlierLimit")}
            value={kept.historyDays}
            disabled={held}
            onChange={(e) =>
              run("earlier", keepHistoryDays(Number(e.target.value)), (days) => {
                amend((was) => ({ ...was, historyDays: days }));
                tell({ card: "earlier", text: t("settingsKept") });
              })
            }
            className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
          >
            {HISTORY_DAYS.map((days) => (
              <option key={days} value={days}>
                {days === 1 ? t("earlierOneDay") : fill("earlierDays", String(days))}
              </option>
            ))}
          </select>
        </Line>
      </div>
    </>
  );
}
