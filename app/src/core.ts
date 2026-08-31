import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

export type Status = "open" | "done" | "dropped";

export type Priority = "do" | "decide" | "delegate" | "minor" | "unset";

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

export interface Volume {
  steps?: number;
  steps_done?: number;
  journal?: number;
  described?: boolean;
  prose?: number;
  refs?: number;
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
  repeat?: Repeat;
  after?: string;
  completed_at?: string;
  hidden?: boolean;
  volume?: Volume;
  created_by?: string;
  filled?: boolean;
  source?: string;
  closed_in?: string;
}

export interface Repeat {
  from: "due" | "done";
  each: { every: number; unit: "day" | "week" | "month" | "year" };
  until?: string | null;
}

export interface List {
  id: string;
  name: string;
  order: string;
  color?: string;
  icon?: string;
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
  refs: string[];
  counts: Record<string, number>;
  locale?: string;
}

export type Mark = "date" | "deadline" | "list" | "tag" | "priority" | "repeat";

export type Certainty = "sure" | "assumed";

export interface Span {
  from: number;
  to: number;
  mark: Mark;
  certainty: Certainty;
}

export interface Offer {
  spans: Span[];
  date: DateSpec;
  title: string;
}

export interface Parsed {
  title: string;
  date?: DateSpec | null;
  deadline?: DateSpec | null;
  priority?: Priority | null;
  tags: string[];
  list?: string | null;
  repeat?: Repeat | null;
  spans: Span[];
  offers: Offer[];
}

export type Reading = "story" | "routine" | "trace";

export type Chapter =
  | { chapter: "born"; title: string }
  | { chapter: "retitled"; from: string; to: string }
  | { chapter: "dated"; from?: DateSpec | null; to?: DateSpec | null }
  | { chapter: "bounded"; from?: DateSpec | null; to?: DateSpec | null }
  | { chapter: "placed"; from: Priority; to: Priority }
  | { chapter: "filed"; from?: string | null; to?: string | null }
  | { chapter: "tagged"; added: string[]; gone: string[] }
  | { chapter: "cadenced"; from?: Repeat | null; to?: Repeat | null }
  | { chapter: "described"; emptied: boolean }
  | { chapter: "wrote"; body: string }
  | { chapter: "rewrote"; body: string }
  | { chapter: "planned"; text: string }
  | { chapter: "ticked"; text: string }
  | { chapter: "unticked"; text: string }
  | { chapter: "reworded"; from: string; to: string }
  | { chapter: "unplanned"; text: string }
  | { chapter: "closed" }
  | { chapter: "dropped" }
  | { chapter: "reopened" };

export type Page = { n: number; at: string; by: string; undoing?: boolean } & Chapter;

export interface Story {
  id: string;
  pages: Page[];
}

export interface Turn {
  id: string;
  status: Status;
  due?: DateSpec;
  closed?: string;
  late?: number;
  gaps?: string[];
  told?: boolean;
  filled?: boolean;
  zone?: string;
}

export interface Series {
  last: string;
  title: string;
  list?: string;
  tags?: string[];
  repeat?: Repeat;
  turns: Turn[];
  kept: number;
  owed: number;
  dropped: number;
  open: number;
  skipped: number;
  streak: number;
  longest: number;
  measurable: boolean;
}

export interface View {
  archive?: boolean;
  reading?: Reading;
  everything?: boolean;
  inbox?: boolean;
  list?: string;
  lists?: string[];
  tags?: string[];
  tagged?: boolean;
  hidden?: boolean;
  window?: "today" | "upcoming" | "overdue" | "undated";
  repeating?: boolean;
}

export const snapshot = (view?: View): Promise<Snapshot> => invoke("snapshot", { view });
export type Scope = "either" | "open" | "archived";

export interface Sighting {
  id: string;
  title: string;
  line: string;
  archived: boolean;
}

export interface Found {
  tasks: Task[];
  papers: Sighting[];
  total: number;
}

export const search = (query: string, scope?: Scope): Promise<Found> =>
  invoke("search", { query, scope });
export const read = (text: string): Promise<Parsed> =>
  invoke("read", { text, locale: navigator.language });
export interface Edits {
  noDate?: boolean;
  noDeadline?: boolean;
  noList?: boolean;
  noPriority?: boolean;
  noRepeat?: boolean;
  noTags?: string[];
  date?: string;
  deadline?: string;
  priority?: Priority;
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
  listNamed?: string;
  inbox?: boolean;
  description?: string;
  remind?: string;
  unremind?: string;
  repeat?: Repeat;
  noRepeat?: boolean;
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
export const complete = (id: string, also?: string[]): Promise<Task> =>
  invoke("complete", { id, also });
export const owed = (id: string): Promise<string[]> => invoke("owed", { id });

export interface Trace {
  kind: "doc" | "file" | "link" | "named";
  target: string;
  label?: string;
  away?: boolean;
  gone?: boolean;
  bytes?: number;
}

export const taskLeft = (id: string): Promise<Trace[]> => invoke("task_left", { id });

export const taskStory = (id: string): Promise<Story> => invoke("task_story", { id });

export const taskSeries = (id: string): Promise<Series | null> => invoke("task_series", { id });

export const allRoutines = (): Promise<Series[]> => invoke("routines");

export interface Month {
  key: string;
  closed: number;
}

export interface Shape {
  closed: number;
  dropped: number;
  told: number;
  since?: string;
  months: Month[];
}

export const archiveShape = (): Promise<Shape> => invoke("archive_shape");

export interface Agent {
  on: boolean;
  called?: string;
  id?: string;
  filed: number;
}

export const agentState = (): Promise<Agent> => invoke("agent");
export const agentTurn = (on: boolean): Promise<Agent> => invoke("agent_turn", { on });

export const attach = (path: string, label?: string, roomy?: boolean): Promise<string> =>
  invoke("attach", { path, label, roomy });

export const served = (reference: string): Promise<string> => invoke("served", { reference });

export const attached = (reference: string): Promise<number[]> => invoke("attached", { reference });

export const attachExport = (reference: string, into: string): Promise<void> =>
  invoke("attach_export", { reference, into });

export const opened = (reference: string): Promise<void> => invoke("opened", { reference });

export interface Carrying {
  chosen?: string;
  asked: boolean;
  backsUp: boolean;
  last?: string;
  heard?: string;
  loose: number;
  open: number;
  archived: number;
  lists: number;
  attachments: number;
  weight: number;
  backedUpAt?: string;
}

export interface Astray {
  at: string;
  bytes: number;
  when: number;
}

export interface Twins {
  bytes: number;
  at: string[];
}

export interface Machine {
  id: string;
  called: string;
  when: number;
  mine: boolean;
}

export interface Reviewed {
  tasks: number;
  lists: number;
  agrees: boolean;
  loose: number;
  looseBytes: number;
  astray: Astray[];
  stranded: number;
  events: number;
  machines: Machine[];
  logBytes: number;
  docsBytes: number;
  heldBytes: number;
  heldFiles: number;
}

export interface Facts {
  version: string;
  dev: boolean;
  sandbox: string | null;
  locale: string;
  zone: string;
  os: string;
  arch: string;
  webview: string | null;
  store: string;
  devices: number;
  events: number;
  open: number;
  archived: number;
  lists: number;
  tags: number;
  listNames: string[];
  tagNames: string[];
  cache: "agrees" | "stale" | "diverged" | "none";
  attachments: number;
  attachmentBytes: number;
  loose: number;
  looseBytes: number;
  weight: number;
  syncs: boolean;
  shared: boolean;
  backedUpAt: string | null;
  quiet: string[];
  attachUpTo: number;
  inPath: boolean;
  shortcut: string | null;
}

export interface About {
  version: string;
  sandbox: string | null;
  repository: string;
  license: string;
  store: string;
}

export interface Settings {
  quiet: string[];
  attachUpTo: number;
  locale?: string;
}

export interface Logs {
  at: string;
  bytes: number;
  lines: string[];
}

export const logs = (most: number): Promise<Logs> => invoke("logs", { most });
export const noteTrouble = (code: string): Promise<void> => invoke("note_trouble", { code });

export type Route = "store" | "brew" | "brewCli" | "download";

export interface Ready {
  version: string;
  route: Route;
  package: string | null;
  installs: boolean;
}

export interface Underway {
  stage: "getting" | "installing";
  far: number;
}

export const updateReady = (nowPlease?: boolean): Promise<Ready | null> =>
  invoke("update_ready", { nowPlease });

export const updateInstall = (): Promise<void> => invoke("update_install");

export const noteBreak = (kind: string, frames: string): Promise<void> =>
  invoke("note_break", { kind, frames });

export const settings = (): Promise<Settings> => invoke("settings");
export const keepSettings = (settings: Settings): Promise<Settings> =>
  invoke("keep_settings", { settings });

export const about = (): Promise<About> => invoke("about");
export const facts = (names: boolean, paths: boolean): Promise<Facts> =>
  invoke("facts", { names, paths });
export const keepReport = (at: string, text: string, logs: boolean): Promise<void> =>
  invoke("keep_report", { at, text, logs });
export const rebuild = (): Promise<void> => invoke("rebuild");
export const checked = (): Promise<Reviewed> => invoke("checked");
export const twinned = (): Promise<Twins[]> => invoke("twinned");
export const syncState = (): Promise<Carrying> => invoke("sync_state");
export const chooseSync = (dest?: string): Promise<void> => invoke("choose_sync", { dest });
export type Carried = "came" | "sent" | "both" | "same" | "busy";

export interface Settled {
  carried: Carried;
  undecided: string[];
  unreadable: string[];
  astray: string[];
  joined: string[];
}

export const syncNow = (way?: "push" | "pull" | "again"): Promise<Settled> =>
  invoke("sync_now", { way });
export const convertPaper = (id: string, body: string): Promise<void> =>
  invoke("convert_paper", { id, body });
export interface Rift {
  was: string[];
  mine: string[];
  theirs: string[];
}

export type Pick = "mine" | "theirs" | "both";

export interface Torn {
  rifts: Rift[];
  print: string;
}

export const paperRifts = (id: string): Promise<Torn> => invoke("paper_rifts", { id });

export const weavePaper = (id: string, picks: Pick[], print: string): Promise<void> =>
  invoke("weave_paper", { id, picks, print });

export const settlePaper = (
  id: string,
  keep: "mine" | "theirs" | "both",
  marked?: string,
): Promise<string | null> => invoke("settle_paper", { id, keep, marked });

export interface Reach {
  shipped: boolean;
  onPath: boolean;
  withinReach: boolean;
  at?: string;
  binary?: string;
  through?: string;
}

export const reachable = (): Promise<Reach> => invoke("reachable");

export interface Wired {
  id: string;
  name: string;
  at: string;
  wired: boolean;
  astray: boolean;
  points?: string;
}

export const seenAgents = (): Promise<Wired[]> => invoke("wiring");
export const wireAgent = (id: string): Promise<Wired[]> => invoke("wire", { id });
export const unwireAgent = (id: string): Promise<Wired[]> => invoke("unwire", { id });
export const reachFor = (wanted: boolean): Promise<Reach> => invoke("reach_for", { wanted });

export interface Waking {
  offered: boolean;
  wakes: boolean;
  theirs: boolean;
}

export const waking = (): Promise<Waking> => invoke("waking");
export const wakeFor = (wanted: boolean): Promise<Waking> => invoke("wake_for", { wanted });

export interface Settling {
  ran: boolean;
  brought: boolean;
  agrees: boolean;
  was?: string;
  stuck?: { code: string; name?: string };
}

export const settleIn = (): Promise<Settling> => invoke("settle_in");

export const shortcut = (): Promise<string | null> => invoke("shortcut");

export const closeWindow = (how?: "hide" | "quit", remember?: boolean): Promise<void> =>
  invoke("close_window", { how, remember });
export const keepLocale = (locale?: string): Promise<string | null> =>
  invoke("keep_locale", { locale });
export const keepClosing = (how: "hide" | "quit"): Promise<void> => invoke("keep_closing", { how });
export const guide = (): Promise<Doc> => invoke("guide");
export const backUp = (into: string): Promise<number> => invoke("back_up", { into });
export const retireAttachment = (reference: string): Promise<void> =>
  invoke("retire_attachment", { reference });
export const removeMachine = (id: string): Promise<void> => invoke("remove_machine", { id });
export const joinThem = (into: string): Promise<number> => invoke("join_them", { into });

export const takeOver = (into: string): Promise<number> => invoke("take_over", { into });

export const mergeStores = (into: string): Promise<boolean> => invoke("merge_stores", { into });

export type Kin = "sameLineage" | "clash" | "unsure" | "strangers";

export const syncKin = (): Promise<Kin> => invoke("sync_kin");
export const restore = (from: string): Promise<number> => invoke("restore", { from });

export const revealed = (path: string): Promise<void> => invoke("revealed", { path });

export const copied = (text: string): Promise<void> => writeText(text);

export const weighs = (reference: string): Promise<number> => invoke("weighs", { reference });

export const roomy = (): Promise<number> => invoke("roomy");
export const reopen = (id: string): Promise<Task> => invoke("reopen", { id });
export const discard = (id: string): Promise<Task> => invoke("discard", { id });
export const erase = (id: string): Promise<void> => invoke("erase", { id });

export interface Doc {
  id: string;
  title: string;
}

/// Only to stop the window offering what the core would refuse; depth.test.ts pins the two together.
export const DEEPEST = 4;

export const FOLDER_NAME_AT_MOST = 40;

export interface Folded {
  id: string;
  name: string;
  parent: string | null;
  icon: string | null;
  color?: string | null;
  holds: number;
}

export type Paper = "a4" | "letter" | "tabloid";

export interface Filed {
  id: string;
  file: string;
  title: string;
  folder: string | null;
  archived: boolean;
  gone?: boolean;
}

export interface Papers {
  folders: Folded[];
  docs: Filed[];
}

export interface DocFacts {
  made: number | null;
  wrote: number | null;
  bytes: number;
}

export const docFacts = (id: string): Promise<DocFacts> => invoke("doc_facts", { id });

export const docs = (): Promise<Papers> => invoke("docs");
export const folderAdd = (name: string, parent?: string, icon?: string): Promise<void> =>
  invoke("folder_add", { name, parent, icon });
export const folderRename = (id: string, name: string): Promise<void> =>
  invoke("folder_rename", { id, name });
export const folderLook = (id: string, icon?: string, color?: string): Promise<void> =>
  invoke("folder_look", { id, icon, color });
export const folderDrop = (id: string): Promise<void> => invoke("folder_drop", { id });
export const docFile = (id: string, folder?: string): Promise<void> =>
  invoke("doc_file", { id, folder });
export const docRead = (id: string): Promise<string> => invoke("doc_read", { id });
export const docWrite = (id: string, body: string, anyway?: boolean): Promise<Doc> =>
  invoke("doc_write", { id, body, anyway });
export const folderFile = (id: string, parent?: string): Promise<void> =>
  invoke("folder_file", { id, parent });

export const printed = (): Promise<void> => invoke("printed");

export const keepPdf = (at: string, bytes: number[]): Promise<void> =>
  invoke("keep_pdf", { at, bytes });

export const parted = (): Promise<void> => invoke("parted");
export const sow = (priority?: Priority): Promise<void> => invoke("sow", { priority });

export const docAway = (id: string, away: boolean): Promise<void> =>
  invoke("doc_away", { id, away });

export const docCopy = (id: string): Promise<Doc> => invoke("doc_copy", { id });

export const docExport = (id: string, into: string): Promise<number> =>
  invoke("doc_export", { id, into });
export const docImport = (from: string, folder?: string): Promise<Doc> =>
  invoke("doc_import", { from, folder });

export const docNew = (folder?: string): Promise<Doc> => invoke("doc_new", { folder });
export const docDrop = (id: string): Promise<void> => invoke("doc_drop", { id });

export const icons = (): Promise<string[]> => invoke("icons");
export const families = (): Promise<[string, number][]> => invoke("families");
export const listAdd = (name: string, icon?: string, color?: string): Promise<List> =>
  invoke("list_add", { name, icon, color });
export const listLook = (id: string, icon?: string, color?: string): Promise<List> =>
  invoke("list_look", { id, icon, color });
export const listRename = (id: string, name: string): Promise<List> =>
  invoke("list_rename", { id, name });
export const listDrop = (id: string): Promise<void> => invoke("list_drop", { id });
