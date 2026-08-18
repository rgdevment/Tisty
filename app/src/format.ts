import type { DateSpec, Repeat } from "./core";
import { fill, locale, t } from "./locales";

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

export function stamped(iso: string, now = new Date()): string {
  const at = new Date(iso);
  const time = formats().clock.format(at);
  return daysFrom(iso, now) === 0 ? time : `${formats().day.format(at)} ${time}`;
}

export const todayLong = (now = new Date()): string =>
  new Intl.DateTimeFormat(locale(), {
    weekday: "long",
    day: "numeric",
    month: "long",
  }).format(now);

export const daysFrom = (iso: string, now = new Date()): number =>
  Math.round((midnight(new Date(iso)) - midnight(now)) / 86_400_000);

export function whenLabel(spec: DateSpec, now = new Date()): string {
  const { day, clock, relative } = formats();
  const at = new Date(spec.at);
  const away = daysFrom(spec.at, now);
  const named = Math.abs(away) <= 1 ? relative.format(away, "day") : day.format(at);
  return spec.has_time ? `${named} ${clock.format(at)}` : named;
}

export function clockOf(spec: DateSpec): string {
  return spec.has_time ? formats().clock.format(new Date(spec.at)) : "";
}

export const isOverdue = (spec: DateSpec, now = new Date()): boolean => daysFrom(spec.at, now) < 0;

export function monthOf(iso?: string, now = new Date()): string {
  if (!iso) return "";
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return "";

  const here = at.getFullYear() === now.getFullYear();
  return months(here).format(at);
}

let named: { for: string; here: Intl.DateTimeFormat; away: Intl.DateTimeFormat } | null = null;

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

export const isToday = (spec: DateSpec, now = new Date()): boolean => daysFrom(spec.at, now) === 0;

export function cadence(repeat: Repeat): string {
  const { every, unit } = repeat.each;
  const one = every === 1;
  const word = t(
    unit === "day"
      ? one
        ? "aDay"
        : "days"
      : unit === "week"
        ? one
          ? "aWeek"
          : "weeks"
        : unit === "month"
          ? one
            ? "aMonth"
            : "months"
          : one
            ? "aYear"
            : "years",
  );
  const said = one ? fill("everyOne", word) : fill("everyMany", `${every} ${word}`);
  return repeat.until ? `${said} ${fill("untilDay", lastDay(repeat.until))}` : said;
}

function lastDay(iso: string): string {
  const at = new Date(`${iso}T00:00:00`);
  return new Intl.DateTimeFormat(locale(), { day: "numeric", month: "short" }).format(at);
}

export function bandOf(spec: DateSpec | undefined, now = new Date()): string {
  if (!spec) return t("someday");
  const away = daysFrom(spec.at, now);
  if (away < 0) return t("overdue");
  if (away === 0) return t("today");
  if (away === 1) return t("tomorrow");
  return formats().day.format(new Date(spec.at));
}

const here = (): string => canonical(Intl.DateTimeFormat().resolvedOptions().timeZone);

const settled = new Map<string, string>();

function canonical(tz: string): string {
  let known = settled.get(tz);
  if (known === undefined) {
    try {
      known = new Intl.DateTimeFormat("en", { timeZone: tz }).resolvedOptions().timeZone;
    } catch {
      known = tz;
    }
    settled.set(tz, known);
  }
  return known;
}

const zones = new Map<string, boolean>();

function usable(tz?: string): tz is string {
  if (!tz) return false;
  const known = zones.get(tz);
  if (known !== undefined) return known;
  try {
    new Intl.DateTimeFormat("en", { timeZone: tz });
    zones.set(tz, true);
    return true;
  } catch {
    zones.set(tz, false);
    return false;
  }
}

const dated = new Map<string, Intl.DateTimeFormat>();

function inZone(tz: string, shape: Intl.DateTimeFormatOptions, tag: string): Intl.DateTimeFormat {
  const key = `${locale()}|${tz}|${tag}`;
  let made = dated.get(key);
  if (!made) {
    made = new Intl.DateTimeFormat(locale(), { ...shape, timeZone: tz });
    dated.set(key, made);
  }
  return made;
}

const days = new Map<string, Intl.DateTimeFormat>();

const dayIn = (at: Date, tz: string): string => {
  let made = days.get(tz);
  if (!made) {
    made = new Intl.DateTimeFormat("en-CA", {
      timeZone: tz,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    });
    days.set(tz, made);
  }
  return made.format(at);
};

export function wroteAt(at: string, tz?: string, now = new Date(), reader = here()): string {
  const when = new Date(at);
  if (Number.isNaN(when.getTime())) return "";
  if (!usable(tz)) return stamped(at, now);

  const away = Math.round(
    (Date.parse(`${dayIn(when, tz)}T00:00:00Z`) - Date.parse(`${dayIn(now, tz)}T00:00:00Z`)) /
      86_400_000,
  );
  const named =
    Math.abs(away) <= 1
      ? formats().relative.format(away, "day")
      : inZone(tz, { weekday: "short", day: "numeric", month: "short" }, "day").format(when);
  const clock = inZone(tz, { hour: "2-digit", minute: "2-digit" }, "clock").format(when);

  const same = canonical(tz) === canonical(reader);
  return same ? `${named} ${clock}` : `${named} ${clock} · ${cityOf(tz)}`;
}

const cityOf = (tz: string): string => (tz.split("/").pop() ?? tz).replace(/_/g, " ");

export function weigh(bytes: number): string {
  if (!Number.isFinite(bytes)) return "—";
  const units = ["B", "kB", "MB", "GB", "TB"];
  let step = 0;
  let left = Math.max(0, bytes);
  while (left >= 1000 && step < units.length - 1) {
    left /= 1000;
    step += 1;
  }
  return `${step === 0 ? left : left.toFixed(1)} ${units[step]}`;
}
