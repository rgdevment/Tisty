import type { Task } from "../core";
import { clockOf } from "../format";
import { locale, t } from "../locales";

interface Props {
  tasks: Task[];
  days: number;
  onOpen: (task: string) => void;
}

const HEAVY = 3;

let cached: { for: string; weekday: Intl.DateTimeFormat } | undefined;

function weekday(): Intl.DateTimeFormat {
  const code = locale();
  if (cached?.for !== code) {
    cached = { for: code, weekday: new Intl.DateTimeFormat(code, { weekday: "short" }) };
  }
  return cached.weekday;
}

const stamp = (at: Date): string =>
  `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, "0")}-${String(at.getDate()).padStart(2, "0")}`;

const stretch = (days: number, now: Date): Date[] => {
  const out: Date[] = [];
  for (let i = 1; i <= days; i += 1) {
    out.push(new Date(now.getFullYear(), now.getMonth(), now.getDate() + i));
  }
  return out;
};

const sorted = (all: Task[]): Task[] =>
  [...all].sort((a, b) => {
    const one = a.date?.has_time ? a.date.at : "";
    const two = b.date?.has_time ? b.date.at : "";
    if (one && two) return one.localeCompare(two);
    if (one) return -1;
    if (two) return 1;
    return 0;
  });

export default function Ahead({ tasks, days, onOpen }: Props) {
  const now = new Date();
  const spread = stretch(days, now).map((at) => {
    const key = stamp(at);
    return { at, key, held: sorted(tasks.filter((one) => one.date?.at.slice(0, 10) === key)) };
  });

  if (spread.every((day) => day.held.length === 0)) {
    return <p className="py-px text-[11.5px] leading-snug text-faint">{t("aheadNothing")}</p>;
  }

  return (
    <div className="flex flex-col gap-px">
      {spread.map((day) => (
        <div
          key={day.key}
          className={`flex gap-2 rounded-md px-1.5 py-1 ${
            day.held.length >= HEAVY ? "bg-hue-amber/12" : ""
          }`}
        >
          <p className="w-8 shrink-0 pt-0.5 text-[9px] font-semibold tracking-[0.05em] text-faint uppercase">
            {weekday().format(day.at)}
            <span
              className={`block text-[11.5px] tabular-nums ${
                day.held.length >= HEAVY ? "text-hue-amber" : "text-ink"
              }`}
            >
              {day.at.getDate()}
            </span>
          </p>
          <div className="flex min-w-0 flex-1 flex-col gap-0.5 pt-0.5">
            {day.held.length === 0 ? (
              <span className="text-[10.5px] text-faint italic">{t("aheadFree")}</span>
            ) : (
              day.held.map((one) =>
                one.date?.has_time ? (
                  <button
                    key={one.id}
                    type="button"
                    onClick={() => onOpen(one.id)}
                    className="flex gap-1.5 rounded-[5px] bg-mark-date px-1.5 py-0.5 text-left text-[10.5px] leading-snug hover:brightness-95"
                  >
                    <span className="shrink-0 tabular-nums text-faint">{clockOf(one.date)}</span>
                    <span className="min-w-0 truncate text-ink">{one.title}</span>
                  </button>
                ) : (
                  <button
                    key={one.id}
                    type="button"
                    onClick={() => onOpen(one.id)}
                    className="flex border-l-2 border-line py-0.5 pl-1.5 text-left text-[10.5px] leading-snug text-soft hover:text-ink"
                  >
                    <span className="min-w-0 truncate">{one.title}</span>
                  </button>
                ),
              )
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
