import type { Coming, DateSpec, Habit } from "../core";
import { cadence, clockOf } from "../format";
import { locale, t } from "../locales";
import { laid, paint } from "./Routine";

interface Props {
  coming: Coming[];
  routines: Habit[];
  days: number;
  onOpen: (task: string) => void;
}

export const HEAVY = 3;
export const WEEK = 7;
const BEADS = 5;

export interface Day {
  at: Date;
  key: string;
  held: Coming[];
}

let cached: { for: string; weekday: Intl.DateTimeFormat } | undefined;

export function weekday(): Intl.DateTimeFormat {
  const code = locale();
  if (cached?.for !== code) {
    cached = { for: code, weekday: new Intl.DateTimeFormat(code, { weekday: "short" }) };
  }
  return cached.weekday;
}

const dayOf = (iso: string): string => {
  const at = new Date(`${iso}T12:00:00`);
  return Number.isNaN(at.getTime()) ? iso : `${weekday().format(at)} ${at.getDate()}`;
};

const stamp = (at: Date): string =>
  `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, "0")}-${String(at.getDate()).padStart(2, "0")}`;

const stretch = (days: number, now: Date): Date[] => {
  const out: Date[] = [];
  for (let i = 1; i <= days; i += 1) {
    out.push(new Date(now.getFullYear(), now.getMonth(), now.getDate() + i));
  }
  return out;
};

export const specOf = (one: Coming): DateSpec | undefined =>
  one.due ? one.task.deadline : one.task.date;

const sorted = (all: Coming[]): Coming[] =>
  [...all].sort((a, b) => {
    const one = specOf(a)?.has_time ? (specOf(a)?.at ?? "") : "";
    const two = specOf(b)?.has_time ? (specOf(b)?.at ?? "") : "";
    if (one && two) return one.localeCompare(two);
    if (one) return -1;
    if (two) return 1;
    return 0;
  });

export const spreadOf = (coming: Coming[], days: number, now: Date): Day[] =>
  stretch(days, now).map((at) => {
    const key = stamp(at);
    return { at, key, held: sorted(coming.filter((one) => one.on === key)) };
  });

export default function Ahead({ coming, routines, days, onOpen }: Props) {
  const spread = spreadOf(coming, days, new Date());
  const kept = routines.filter((one) => one.task.repeat);

  const strip =
    kept.length > 0 ? (
      <div className="mb-1 flex flex-col gap-0.5 border-b border-hair pb-1">
        {kept.map((one) => (
          <button
            key={one.task.id}
            type="button"
            onClick={() => onOpen(one.task.id)}
            className="flex items-center gap-1.5 px-1.5 text-left text-[10.5px] leading-snug text-soft hover:text-ink"
          >
            <span className="shrink-0 text-[9px] tracking-[0.05em] text-faint uppercase">
              {one.task.repeat ? cadence(one.task.repeat) : ""}
              {one.on ? ` · ${dayOf(one.on)}` : ""}
            </span>
            <span className="min-w-0 truncate">{one.task.title}</span>
            {one.series && (
              <span className="ml-auto flex shrink-0 items-center gap-1.5">
                <span className="flex gap-px" aria-hidden="true">
                  {laid(one.series, BEADS)
                    .slice(-BEADS)
                    .map((day) => (
                      <span key={day.key} className={`h-1.5 w-1.5 rounded-md ${paint(day.mark)}`} />
                    ))}
                </span>
                <span className="tabular-nums text-ink">{one.series.streak}</span>
              </span>
            )}
          </button>
        ))}
      </div>
    ) : null;

  if (spread.every((day) => day.held.length === 0) && kept.length === 0) {
    return <p className="py-px text-[11.5px] leading-snug text-faint">{t("aheadNothing")}</p>;
  }

  return (
    <div className="flex flex-col gap-px">
      {strip}

      {spread.map((day) => {
        const heavy = day.held.length >= HEAVY;
        const bare = day.held.length === 0;
        return (
          <div
            key={day.key}
            className={`flex gap-2 rounded-md px-1.5 py-0.5 ${heavy ? "bg-hue-amber/10" : ""}`}
          >
            <p
              className={`w-11 shrink-0 truncate pt-px text-[9px] font-semibold tracking-[0.05em] uppercase ${
                bare ? "text-faint/70" : "text-faint"
              }`}
            >
              {weekday().format(day.at)}{" "}
              <span
                className={`tabular-nums ${
                  heavy ? "text-hue-amber" : bare ? "text-faint/70" : "text-ink"
                }`}
              >
                {day.at.getDate()}
              </span>
            </p>
            <div className="flex min-w-0 flex-1 flex-col gap-0.5">
              {day.held.map((one) => {
                const spec = specOf(one);
                const due = one.due;
                return spec?.has_time ? (
                  <button
                    key={`${one.task.id} ${one.on}`}
                    type="button"
                    onClick={() => onOpen(one.task.id)}
                    className={`flex gap-1.5 rounded-md px-1.5 py-0.5 text-left text-[10.5px] leading-snug hover:brightness-95 ${
                      due ? "bg-mark-deadline" : "bg-mark-date"
                    }`}
                  >
                    <span className="shrink-0 tabular-nums text-faint">{clockOf(spec)}</span>
                    <span className="min-w-0 truncate text-ink">{one.task.title}</span>
                  </button>
                ) : (
                  <button
                    key={`${one.task.id} ${one.on}`}
                    type="button"
                    onClick={() => onOpen(one.task.id)}
                    className={`flex border-l-2 py-0.5 pl-1.5 text-left text-[10.5px] leading-snug text-soft hover:text-ink ${
                      due ? "border-hue-amber" : "border-line"
                    }`}
                  >
                    <span className="min-w-0 truncate">{one.task.title}</span>
                  </button>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}
