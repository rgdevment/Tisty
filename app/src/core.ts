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
  hidden?: boolean;
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

export type Mark = "date" | "deadline" | "list" | "tag" | "priority";

/** `assumed` is applied all the same; the window only says so. */
export type Certainty = "sure" | "assumed";

/** Offsets count code points, so slice with `Array.from`, never with indexes. */
export interface Span {
  from: number;
  to: number;
  mark: Mark;
  certainty: Certainty;
}

/** A reading the parser saw and did not take; offered rather than applied. */
export interface Offer {
  spans: Span[];
  date: DateSpec;
  /** What the title becomes if it is taken. */
  title: string;
}

/** What the parser made of the text, without writing anything. */
export interface Parsed {
  title: string;
  date?: DateSpec | null;
  deadline?: DateSpec | null;
  priority?: Priority | null;
  tags: string[];
  list?: string | null;
  spans: Span[];
  offers: Offer[];
}

/** What the sidebar asks for. The core decides which tasks answer. */
export interface View {
  archive?: boolean;
  /** A tag reaches across the archive, unlike every other view. */
  everything?: boolean;
  inbox?: boolean;
  list?: string;
  tags?: string[];
  /** Anything carrying a tag: the tag view with nothing picked. */
  tagged?: boolean;
  hidden?: boolean;
  window?: "today" | "upcoming" | "overdue";
}

export const snapshot = (view?: View): Promise<Snapshot> => invoke("snapshot", { view });
export type Scope = "either" | "open" | "archived";

export const search = (query: string, scope: Scope = "open"): Promise<Task[]> =>
  invoke("search", { query, scope });
export const read = (text: string): Promise<Parsed> =>
  invoke("read", { text, locale: navigator.language });
/** What the chips changed by hand. Same knobs `tisty add` takes as flags. */
export interface Edits {
  noDate?: boolean;
  noDeadline?: boolean;
  noList?: boolean;
  noPriority?: boolean;
  noTags?: string[];
  date?: string;
  deadline?: string;
  priority?: Priority;
  /** The offered reading was accepted, so its words leave the title. */
  takeOffer?: boolean;
}

export const capture = (text: string, view?: View, edits?: Edits): Promise<Task> =>
  invoke("capture", { text, locale: navigator.language, view, edits });
export interface Change {
  title?: string;
  date?: string;
  noDate?: boolean;
  deadline?: string;
  noDeadline?: boolean;
  priority?: Priority;
  addTag?: string;
  untag?: string;
  list?: string;
  inbox?: boolean;
  description?: string;
  remind?: string;
  unremind?: string;
}

export const patch = (id: string, change: Change): Promise<Task> =>
  invoke("patch", { id, change, locale: navigator.language });
export const writeStep = (id: string, text: string, step?: string): Promise<Task> =>
  invoke("write_step", { id, step, text });
export const markStep = (id: string, step: string, done: boolean): Promise<Task> =>
  invoke("mark_step", { id, step, done });
export const dropStep = (id: string, step: string): Promise<Task> =>
  invoke("drop_step", { id, step });
export const writeLog = (id: string, body: string, entry?: string): Promise<Task> =>
  invoke("write_log", { id, entry, body });
export const fold = (id: string, away: boolean): Promise<Task> => invoke("fold", { id, away });
export const complete = (id: string): Promise<Task> => invoke("complete", { id });
export const reopen = (id: string): Promise<Task> => invoke("reopen", { id });
export const discard = (id: string): Promise<Task> => invoke("discard", { id });
