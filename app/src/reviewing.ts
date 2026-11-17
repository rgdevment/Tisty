import { invoke } from "@tauri-apps/api/core";
import type { Task } from "./core";

export interface Reviewing {
  routine: Task | null;
  unlisted: Task[];
}

export interface Nudge {
  title: string;
  tag: string;
  list: string;
  weekday: number;
  hour: number;
  minute: number;
}

export const NUDGE_WEEKDAY = 5;
export const NUDGE_AT = "17:00";

export const reviewOf = (): Promise<Reviewing> => invoke("review_of");

export const reviewRemind = (nudge: Nudge): Promise<Task> => invoke("review_remind", { nudge });
