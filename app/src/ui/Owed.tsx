import { useState } from "react";
import { fill, locale, t } from "../locales";

interface Props {
  days: string[];
  onConfirm: (days: string[]) => void;
}

export default function Owed({ days, onConfirm }: Props) {
  const [taken, setTaken] = useState<string[]>([]);

  const flip = (day: string) =>
    setTaken((held) => (held.includes(day) ? held.filter((one) => one !== day) : [...held, day]));

  return (
    <div className="mx-2 mt-1.5 rounded-[10px] border border-line bg-panel px-3 py-2.5">
      <div className="flex items-center gap-2">
        <p className="text-[12.5px] text-soft">
          <b className="font-semibold text-ink">{t("owedAsk")}</b> {t("owedWhy")}
        </p>
        <button
          type="button"
          aria-label={t("owedSkip")}
          title={t("owedSkip")}
          onClick={() => onConfirm([])}
          className="ml-auto cursor-pointer text-faint hover:text-soft"
        >
          ✕
        </button>
      </div>
      <div className="mt-2.5 flex flex-wrap items-center gap-1.5">
        {days.map((day) => {
          const on = taken.includes(day);
          return (
            <button
              key={day}
              type="button"
              aria-pressed={on}
              onClick={() => flip(day)}
              className={`cursor-pointer rounded-full border px-3 py-[5px] text-[12.5px] ${
                on
                  ? "border-accent bg-accent font-semibold text-white"
                  : "border-line bg-bg text-soft"
              }`}
            >
              {dayLabel(day)}
            </button>
          );
        })}
        {taken.length > 0 && (
          <button
            type="button"
            onClick={() => onConfirm(taken)}
            className="ml-auto cursor-pointer rounded-lg border border-accent bg-accent px-3 py-[5px] text-[12.5px] font-semibold text-white"
          >
            {fill("owedFill", String(taken.length))}
          </button>
        )}
      </div>
    </div>
  );
}

/// A civil date read as UTC lands on the day before west of Greenwich.
function dayLabel(day: string): string {
  const [year, month, date] = day.split("-").map(Number);
  const at = new Date(year, month - 1, date);
  const now = new Date();
  const away = Math.round(
    (at.getTime() - new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime()) /
      86_400_000,
  );
  if (away === 0) return t("owedToday");
  if (away === -1) return t("owedYesterday");
  return new Intl.DateTimeFormat(locale(), {
    weekday: "short",
    day: "numeric",
  }).format(at);
}
