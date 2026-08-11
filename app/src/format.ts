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

/** A moment, not a day: syncing happens several times an hour. */
export function stamped(iso: string, now = new Date()): string {
  const at = new Date(iso);
  const time = formats().clock.format(at);
  return daysFrom(iso, now) === 0 ? time : `${formats().day.format(at)} ${time}`;
}

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

/// The archive is read by month: «fue por marzo». The year only shows once it
/// stops being the current one.
export function monthOf(iso?: string, now = new Date()): string {
  if (!iso) return "";
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return "";

  const here = at.getFullYear() === now.getFullYear();
  return months(here).format(at);
}

let named: { for: string; here: Intl.DateTimeFormat; away: Intl.DateTimeFormat } | null = null;

/// Built once: the archive calls this per task, and a formatter costs more than
/// the comparison it feeds.
function months(here: boolean): Intl.DateTimeFormat {
  const code = locale();
  if (named?.for !== code) {
    named = {
      for: code,
      here: new Intl.DateTimeFormat(code, { month: "long" }),
      away: new Intl.DateTimeFormat(code, { month: "long", year: "numeric" }),
    };
  }
  return here ? named.here : named.away;
}

export const isToday = (spec: DateSpec, now = new Date()): boolean =>
  daysFrom(spec.at, now) === 0;
