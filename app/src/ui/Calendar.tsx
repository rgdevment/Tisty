import { useEffect, useRef, useState } from "react";
import { locale, t } from "../locales";

interface Props {
  value?: string;
  inline?: boolean;
  onPick: (iso: string) => void;
  onClear: () => void;
  onClose: () => void;
}

export default function Calendar({ value, inline, onPick, onClear, onClose }: Props) {
  const box = useRef<HTMLDivElement>(null);
  const today = civil(new Date());
  const [month, setMonth] = useState(() => first(value ?? today));

  useEffect(() => {
    const away = (e: MouseEvent) => {
      if (!box.current?.contains(e.target as Node)) onClose();
    };
    const key = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    if (!inline) document.addEventListener("mousedown", away);
    document.addEventListener("keydown", key);
    return () => {
      document.removeEventListener("mousedown", away);
      document.removeEventListener("keydown", key);
    };
  }, [inline, onClose]);

  const { day: named, heading } = formats();

  return (
    <div
      ref={box}
      className={
        inline
          ? "w-full text-[12.5px]"
          : "absolute top-6 left-0 z-10 w-[236px] rounded-[10px] border border-line bg-bg p-2.5 text-[12.5px] shadow-lg"
      }
    >
      <div className="mb-1.5 flex items-center">
        <Step to={-1} on={month} go={setMonth} />
        <span className="flex-1 text-center font-medium text-ink capitalize">
          {heading.format(month)}
        </span>
        <Step to={1} on={month} go={setMonth} />
      </div>

      <div className="grid grid-cols-7 text-center text-[10.5px] text-faint">
        {week().map((day) => (
          <span key={day.getTime()} className="py-1">
            {named.format(day).slice(0, 2)}
          </span>
        ))}
      </div>

      <div className="grid grid-cols-7 gap-px">
        {days(month).map((day) => {
          const iso = civil(day);
          return (
            <button
              key={iso}
              type="button"
              onClick={() => onPick(iso)}
              className={`h-[26px] rounded-md tabular-nums ${
                iso === value
                  ? "bg-accent font-medium text-white"
                  : `hover:bg-hover ${day.getMonth() === month.getMonth() ? "text-ink" : "text-faint"} ${
                      iso === today ? "font-semibold text-accent" : ""
                    }`
              }`}
            >
              {day.getDate()}
            </button>
          );
        })}
      </div>

      <div className="mt-1.5 flex gap-1.5 border-t border-hair pt-2">
        {[
          [t("today"), 0],
          [t("tomorrow"), 1],
        ].map(([label, away]) => (
          <button
            key={label}
            type="button"
            onClick={() => onPick(civil(shifted(away as number)))}
            className="flex-1 rounded-md bg-hover py-1 hover:bg-active"
          >
            {label}
          </button>
        ))}
        <button
          type="button"
          onClick={onClear}
          className="flex-1 rounded-md bg-hover py-1 text-soft hover:bg-active"
        >
          {t("remove")}
        </button>
      </div>
    </div>
  );
}

function Step({ to, on, go }: { to: number; on: Date; go: (d: Date) => void }) {
  return (
    <button
      type="button"
      aria-label={t(to < 0 ? "prevMonth" : "nextMonth")}
      onClick={() => go(new Date(on.getFullYear(), on.getMonth() + to, 1))}
      className="flex h-6 w-6 items-center justify-center rounded-md text-faint hover:bg-hover hover:text-ink"
    >
      {to < 0 ? "‹" : "›"}
    </button>
  );
}

let cached: { for: string; day: Intl.DateTimeFormat; heading: Intl.DateTimeFormat } | null = null;

function formats() {
  const code = locale();
  if (cached?.for !== code) {
    cached = {
      for: code,
      day: new Intl.DateTimeFormat(code, { weekday: "short" }),
      heading: new Intl.DateTimeFormat(code, { month: "long", year: "numeric" }),
    };
  }
  return cached;
}

/** First day of the week varies by locale (Sunday in the US, Monday elsewhere). */
function opens(): number {
  const info = new Intl.Locale(locale()) as Intl.Locale & { weekInfo?: { firstDay: number } };
  return (info.weekInfo?.firstDay ?? 1) % 7;
}

const first = (iso: string): Date => {
  const [y, m] = iso.split("-").map(Number);
  return new Date(y, m - 1, 1);
};

const shifted = (days: number): Date => {
  const on = new Date();
  on.setDate(on.getDate() + days);
  return on;
};

/** Built by hand, not `toISOString` — that reports UTC, not the reader's local day. */
function civil(on: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${on.getFullYear()}-${pad(on.getMonth() + 1)}-${pad(on.getDate())}`;
}

function week(): Date[] {
  const from = new Date(2024, 0, 7 + opens());
  return Array.from({ length: 7 }, (_, i) => new Date(2024, 0, from.getDate() + i));
}

/** Six rows always, so the popover does not jump height between months. */
function days(month: Date): Date[] {
  const lead = (month.getDay() - opens() + 7) % 7;
  const from = new Date(month.getFullYear(), month.getMonth(), 1 - lead);
  return Array.from(
    { length: 42 },
    (_, i) => new Date(from.getFullYear(), from.getMonth(), from.getDate() + i),
  );
}
