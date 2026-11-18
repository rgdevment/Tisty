import { invoke } from "@tauri-apps/api/core";
import type { DateSpec, Repeat, Settings } from "./core";
import type { t } from "./locales";

export type CalendarChoice = NonNullable<Settings["calendar"]>;

export const keepCalendar = (calendar?: CalendarChoice): Promise<CalendarChoice | null> =>
  invoke("keep_calendar", { calendar });
export const saveToCalendar = (id: string, calendar?: CalendarChoice): Promise<boolean> =>
  invoke("save_to_calendar", { id, calendar });

export interface Offered {
  choice: CalendarChoice;
  label: Parameters<typeof t>[0];
  away?: boolean;
}

export const offered = (onMac: boolean): Offered[] => [
  { choice: "google", label: "calendarGoogle" },
  { choice: "outlook", label: "calendarOutlook" },
  { choice: "outlook-work", label: "calendarOutlookWork" },
  onMac
    ? { choice: "apple", label: "calendarApple" }
    : { choice: "apple", label: "calendarAppleAway", away: true },
];

export type IcsOdd = "untitled" | "undated" | "moved" | "cancelled" | "repeat" | "zone" | "past";

export interface IcsFound {
  at: number;
  kind: "event" | "todo";
  title: string;
  date?: DateSpec | null;
  deadline?: DateSpec | null;
  repeat?: Repeat | null;
  reminder?: DateSpec | null;
  place: string;
  odd: IcsOdd[];
  done: boolean;
  already: boolean;
  takeable: boolean;
}

export interface IcsBrought {
  found: IcsFound[];
  invitation: boolean;
  stamp: string;
}

export const icsWaiting = (): Promise<string[]> => invoke("ics_waiting");
export const icsRead = (path: string): Promise<IcsBrought> => invoke("ics_read", { path });
export const icsTake = (
  path: string,
  stamp: string,
  picked: number[],
  list: string | null,
): Promise<{ made: number; missed: number }> => invoke("ics_take", { path, stamp, picked, list });
