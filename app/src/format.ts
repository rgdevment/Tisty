import type { DateSpec } from "./core";
import { locale } from "./locales";

type Formats = {
  day: Intl.DateTimeFormat;
  clock: Intl.DateTimeFormat;
  relative: Intl.RelativeTimeFormat;
};

let cached: (Formats & { for: string }) | null = null;

function formats(): Formats {
  const code = locale();
  if (cached?.for !== code) {
    cached = {
      for: code,
      day: new Intl.DateTimeFormat(code, { weekday: "short", day: "numeric", month: "short" }),
      clock: new Intl.DateTimeFormat(code, { hour: "2-digit", minute: "2-digit" }),
      relative: new Intl.RelativeTimeFormat(code, { numeric: "auto" }),
    };
  }
  return cached;
}

const midnight = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();

/** Whole days apart on the reader's calendar, not 24-hour spans. */
export const daysFrom = (iso: string, now = new Date()): number =>
  Math.round((midnight(new Date(iso)) - midnight(now)) / 86_400_000);

export function whenLabel(spec: DateSpec, now = new Date()): string {
  const { day, clock, relative } = formats();
  const at = new Date(spec.at);
  const away = daysFrom(spec.at, now);
  const named = Math.abs(away) <= 1 ? relative.format(away, "day") : day.format(at);
  return spec.has_time ? `${named} ${clock.format(at)}` : named;
}

export const isOverdue = (spec: DateSpec, now = new Date()): boolean =>
  daysFrom(spec.at, now) < 0;

export const isToday = (spec: DateSpec, now = new Date()): boolean =>
  daysFrom(spec.at, now) === 0;
