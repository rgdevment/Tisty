import type { Settings } from "../core";
import { HISTORY_DAYS, keepHistoryDays } from "../earlier";
import { fill, t } from "../locales";
import type { Run, Word } from "./Card";
import { Band, Line, off } from "./Rows";

interface Props {
  kept: Settings;
  setKept: (kept: Settings) => void;
  held: boolean;
  said?: Word;
  trouble?: Word;
  run: Run;
}

export default function EarlierLimit({ kept, setKept, held, said, trouble, run }: Props) {
  return (
    <>
      <Band label={t("earlierBand")} />
      <div className="border-t border-hair">
        <Line
          title={t("earlierLimit")}
          why={t("earlierLimitWhy")}
          which="earlier"
          said={said}
          trouble={trouble}
        >
          <select
            aria-label={t("earlierLimit")}
            value={kept.historyDays}
            disabled={held}
            onChange={(e) =>
              run("earlier", keepHistoryDays(Number(e.target.value)), (days) =>
                setKept({ ...kept, historyDays: days }),
              )
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
