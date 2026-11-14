import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "./core";
import type { t } from "./locales";

export type CalendarChoice = NonNullable<Settings["calendar"]>;

export const keepCalendar = (calendar?: CalendarChoice): Promise<CalendarChoice | null> =>
  invoke("keep_calendar", { calendar });
export const saveToCalendar = (id: string): Promise<boolean> => invoke("save_to_calendar", { id });

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
