import { invoke } from "@tauri-apps/api/core";

export type Status = "open" | "done" | "dropped";

/** 1 is the most urgent; the core sorts ascending. */
export type Priority = 1 | 2 | 3 | 4;

export interface DateSpec {
  at: string;
  tz: string;
  floating: boolean;
  has_time: boolean;
}

export interface Step {
  id: string;
  text: string;
  done: boolean;
  order: string;
}

export interface LogEntry {
  id: string;
  at: string;
  tz?: string;
  body: string;
}

/** Counts kept apart from the vectors, so a summary can report a body it never loaded. */
export interface Volume {
  steps?: number;
  steps_done?: number;
  journal?: number;
  described?: boolean;
}

export interface Task {
  id: string;
  title: string;
  status: Status;
  priority: Priority;
  order: string;
  description?: string;
  log?: LogEntry[];
  steps?: Step[];
  date?: DateSpec;
  deadline?: DateSpec;
  list?: string;
  tags?: string[];
  reminders?: DateSpec[];
  completed_at?: string;
  volume?: Volume;
}

export interface List {
  id: string;
  name: string;
  order: string;
  color?: string;
  archived?: boolean;
}

export interface Snapshot {
  tasks: Task[];
  lists: List[];
}

export const snapshot = (): Promise<Snapshot> => invoke("snapshot");
export const complete = (id: string): Promise<void> => invoke("complete", { id });
export const reopen = (id: string): Promise<void> => invoke("reopen", { id });
