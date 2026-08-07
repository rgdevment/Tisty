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

export interface Counted {
  tag: string;
  tasks: number;
}

export interface Snapshot {
  tasks: Task[];
  lists: List[];
  tags: Counted[];
  /** Per view and per list id, counted by the core with the same filter. */
  counts: Record<string, number>;
  locale?: string;
}

/** What the parser made of the text, without writing anything. */
export interface Parsed {
  title: string;
  date?: DateSpec | null;
  deadline?: DateSpec | null;
  priority?: Priority | null;
  tags: string[];
  list?: string | null;
}

/** What the sidebar asks for. The core decides which tasks answer. */
export interface View {
  archive?: boolean;
  /** A tag reaches across the archive, unlike every other view. */
  everything?: boolean;
  inbox?: boolean;
  list?: string;
  tags?: string[];
  window?: "today" | "upcoming" | "overdue";
}

export const snapshot = (view?: View): Promise<Snapshot> => invoke("snapshot", { view });
export type Scope = "either" | "open" | "archived";

export const search = (query: string, scope: Scope = "open"): Promise<Task[]> =>
  invoke("search", { query, scope });
export const read = (text: string): Promise<Parsed> =>
  invoke("read", { text, locale: navigator.language });
export const capture = (text: string, view?: View): Promise<Task> =>
  invoke("capture", { text, locale: navigator.language, view });
export const complete = (id: string): Promise<void> => invoke("complete", { id });
export const reopen = (id: string): Promise<void> => invoke("reopen", { id });
