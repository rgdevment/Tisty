import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { Paced, Standing } from "./deadlines";
import { speakOf } from "./locales";
import { settled } from "./saving";

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
  said?: boolean;
}

export interface LogEntry {
  id: string;
  at: string;
  tz?: string;
  body: string;
  by?: string;
  via?: string;
}

export interface Volume {
  steps?: number;
  steps_done?: number;
  journal?: number;
  described?: boolean;
  prose?: number;
  refs?: number;
  grew?: number;
}

export interface Resolved {
  at: string;
  by: string;
  entry: string;
  via?: string;
  drop?: boolean;
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
  part_of?: string;
  completed_at?: string;
  hidden?: boolean;
  volume?: Volume;
  created_by?: string;
  resolved?: Resolved;
  filled?: boolean;
  source?: string;
  closed_in?: string;
  read_as?: Reading;
  open_to_agents?: boolean;
  created_via?: string;
}

export const STORY_AT = 3;

// The same sum the core makes of a task's volume: prose, then steps and references by bucket.
export const weightOf = (task: Task): number => {
  const steps = task.volume?.steps ?? 0;
  const refs = task.volume?.refs ?? 0;
  const plan = steps >= 8 ? 2 : steps >= 3 ? 1 : 0;
  const links = refs >= 3 ? 2 : refs >= 1 ? 1 : 0;
  const visits = Math.max(0, (task.volume?.grew ?? 0) - 1);
  return Math.max((task.volume?.prose ?? 0) + plan + links, visits);
};

export const readingOf = (task: Task): Reading => {
  if (task.repeat || task.after) return "routine";
  if (task.read_as === "story" || task.read_as === "trace") return task.read_as;
  return weightOf(task) >= STORY_AT ? "story" : "trace";
};

// Only a closed trace goes; a story is hidden, and converting it is the deliberate step.
export const erasable = (task: Task): boolean =>
  task.status !== "open" && readingOf(task) === "trace";

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
  docs: number;
}

export interface Coming {
  task: Task;
  on: string;
  due: boolean;
}

export interface Habit {
  task: Task;
  series?: Series;
  on?: string;
}

export interface Snapshot {
  tasks: Task[];
  total: number;
  soonest: Record<string, Task[]>;
  ahead: Coming[];
  routines: Habit[];
  lists: List[];
  shelved?: List[];
  every?: List[];
  tags: Counted[];
  refs: string[];
  counts: Record<string, number>;
  locale?: string;
  agents: Record<string, string>;
  agent_tag?: string;
  hosts?: Record<string, string>;
  machines?: Record<string, string>;
  machine_here?: string;
  clients?: Record<string, string>;
  wholes?: Record<string, Whole>;
  standings?: Record<string, Standing>;
  paces?: Record<string, Paced>;
}

export interface Whole {
  title: string;
  open: number;
  closed: number;
  away?: number;
}

export interface Offered {
  id: string;
  title: string;
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
  | { chapter: "sectionadded"; section: string }
  | { chapter: "sectionrenamed"; from: string; to: string }
  | { chapter: "sectionremoved"; section: string }
  | { chapter: "sectionwritten"; section: string }
  | { chapter: "joined"; whole: string; title: string }
  | { chapter: "loosened"; whole: string; title: string }
  | { chapter: "partjoined"; part: string; title: string }
  | { chapter: "partloosened"; part: string; title: string }
  | { chapter: "closed" }
  | { chapter: "dropped" }
  | { chapter: "reopened" }
  | { chapter: "converted"; from?: Reading | null; to?: Reading | null }
  | { chapter: "opened" }
  | { chapter: "shut" };

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
  steps?: number;
  steps_done?: number;
  said?: string[];
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
  most?: number;
  board?: boolean;
  spread?: boolean;
  soonest?: boolean;
}

export const snapshot = (view?: View): Promise<Snapshot> =>
  invoke<Snapshot>("snapshot", { view }).then((one) =>
    one ? { ...one, every: [...one.lists, ...(one.shelved ?? [])] } : one,
  );
export type Scope = "either" | "open" | "archived";

export interface Sighting {
  id: string;
  title: string;
  line: string;
  archived: boolean;
}

export interface Found {
  standings?: Record<string, Standing>;
  tasks: Task[];
  papers: Sighting[];
  total: number;
  wholes?: Record<string, Whole>;
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
export type TaskDoc = { readonly file: string; readonly body: string; readonly held: boolean };
export const taskDocOpen = (id: string): Promise<TaskDoc | null> => invoke("task_doc_open", { id });
export const taskDocMake = (id: string, body?: string): Promise<string> =>
  invoke("task_doc_make", { id, body });
export const taskDocSettled = (
  id: string,
  before: string,
  now: string,
  renamed: readonly [string, string][] = [],
): Promise<Task | null> => invoke("task_doc_settled", { id, before, now, renamed });
export const partsOf = (id: string): Promise<Task[]> => invoke("parts_of", { id });
export type StepDated = { text: string; born?: string | null; done?: string | null };
export const stepDates = (id: string): Promise<StepDated[]> => invoke("task_step_dates", { id });
export const taskDocMerge = (id: string, base: string, mine: string): Promise<boolean> =>
  invoke("task_doc_merge", { id, base, mine });
export const taskPageImport = (from: string, anyway?: boolean): Promise<string | null> =>
  invoke("task_page_import", { from, anyway });
export const taskOf = (id: string): Promise<Task> => invoke("task_of", { id });
export const addPart = (whole: string, title: string): Promise<Task> =>
  invoke("add_part", { whole, title });
export const hang = (id: string, whole: string | null): Promise<Task> =>
  invoke("hang", { id, whole });
export const wholesOffered = (id: string): Promise<Offered[]> => invoke("wholes_offered", { id });
export const fold = (id: string, away: boolean): Promise<Task> => invoke("fold", { id, away });
export const readAs = (id: string, how: "story" | "trace"): Promise<Task> =>
  invoke("read_as", { id, how });
export const openToAgents = (id: string, open: boolean): Promise<Task> =>
  invoke("open_to_agents", { id, open });
export const stillOpen = (id: string): Promise<Task> => invoke("still_open", { id });
export const stepsSaid = (id: string, keep: boolean, steps: string[]): Promise<Task> =>
  invoke("steps_said", { id, steps, keep });
// What this round wrote has to reach the page before closing reads it.
export const complete = async (id: string, also?: string[], summary?: string): Promise<Task> => {
  await settled();
  return invoke("complete", { id, also, summary });
};
export const owed = (id: string): Promise<string[]> => invoke("owed", { id });

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
  code?: string;
}

export const agentState = (): Promise<Agent> => invoke("agent");
export const agentTurn = (on: boolean): Promise<Agent> => invoke("agent_turn", { on });

export const attach = (path: string, label?: string, shape?: "full" | "light"): Promise<string> =>
  invoke("attach", { path, label, shape });
export const attachPasted = (name: string, kind: string, bytes: Uint8Array): Promise<string> =>
  invoke("attach_pasted", bytes, {
    headers: { "x-name": encodeURIComponent(name), "x-kind": encodeURIComponent(kind) },
  });

export const openLink = (url: string): Promise<void> => invoke("open_link", { url });
export const widgetLend = (body: string): Promise<string> => invoke("widget_lend", { body });
export type Borrowed = { id: string; whole: boolean };
export const widgetLendKept = (reference: string): Promise<Borrowed> =>
  invoke("widget_lend_kept", { reference });
export const widgetTakeBack = (id: string): Promise<void> => invoke("widget_take_back", { id });

export type Lent = { source: string } | { reference: string };
export type Viewed = { id: string; file: string; title: string; place: string };
export type Showing =
  | { kind: "file"; name: string; place: string; path: string; body: string }
  | {
      kind: "missed";
      name: string;
      place: string;
      path: string;
      refusal: { code: string; name?: string };
    }
  | { kind: "doc"; paper: Viewed; body: string }
  | { kind: "widget"; lent: Lent; from: Viewed | null }
  | { kind: "diagram"; paper: Viewed; source: string };
export const viewShown = (): Promise<Showing> => invoke("view_shown");
export const viewFacts = (): Promise<DocFacts> => invoke("view_facts");
export const viewImport = (open: boolean): Promise<Viewed & Left> =>
  invoke("view_import", { open });
export const viewGo = (file: string): Promise<void> => invoke("view_go", { file });
export const viewWidget = (lent: Lent, from?: Viewed): Promise<void> =>
  invoke("view_widget", { lent, from: from ?? null });
export const viewDoc = (paper: Viewed): Promise<void> => invoke("view_doc", { paper });

export const served = (reference: string): Promise<string> => invoke("served", { reference });
export const drawsHeicHere = (): Promise<boolean> => invoke("draws_heic");

export const attached = (reference: string): Promise<number[]> => invoke("attached", { reference });
export const keptHere = (reference: string): Promise<boolean> => invoke("kept_here", { reference });
export const copyAttached = (reference: string, to: string): Promise<void> =>
  invoke("copied", { reference, to });

export const opened = (reference: string): Promise<void> => invoke("opened", { reference });

export type Keeper = "cloud" | "away" | "plain";

export interface Offering {
  key: string;
  named: string;
  at?: string;
  into?: string;
}

export interface Told {
  keeper: Keeper;
  named?: string;
  into: string;
}

export const keepers = (): Promise<Offering[]> => invoke("keepers");
export const keeperOf = (at: string): Promise<Told> => invoke("keeper_of", { at });
export interface Strays {
  adrift: number;
  unreadable: boolean;
}
export const straysAt = (at: string): Promise<Strays> => invoke("strays_at", { at });
export const makeRoom = (at: string): Promise<void> => invoke("make_room", { at });
export const sowLists = (): Promise<void> => invoke("sow_lists");

export interface Glimpse {
  title?: string;
  said?: string;
  shot?: string;
}

export const glimpseKept = (at: string): Promise<Glimpse | null> => invoke("glimpse_kept", { at });
export const glimpseFetch = (at: string): Promise<Glimpse | null> =>
  invoke("glimpse_fetch", { at });

export interface Carrying {
  chosen?: string;
  keeper?: Keeper;
  keptBy?: string;
  asked: boolean;
  last?: string;
  heard?: string;
  open: number;
  archived: number;
  lists: number;
  attachments: number;
  weight: number;
  carries: number;
  sharedWas?: string;
  sharedWasLater?: boolean;
  later?: boolean;
  mode: "alone" | "folder" | "cloud" | "later";
  cloud?: string | null;
  cloudProvider?: string | null;
  cloudOffered: boolean;
  clouds: string[];
  backedUpAt?: string;
  freesSpace: boolean;
}

export interface Astray {
  at: string;
  bytes: number;
  when: number;
  shared?: boolean;
}

export interface Linked {
  kind: "doc" | "task";
  id: string;
  title: string;
}

export interface Came {
  how?: string | null;
  machine: string;
  mine: boolean;
  when: number;
}

export interface Twin {
  at: string;
  links: Linked[];
  locked?: string | null;
  came?: Came | null;
}

export interface Pair {
  bytes: number;
  stays: string;
  copies: Twin[];
  unread?: string | null;
}

export interface Joined {
  freed: number;
  relinked: number;
  left: string[];
}

export interface Machine {
  id: string;
  called: string;
  name?: string | null;
  os?: string | null;
  when: number;
  since?: number;
  mine: boolean;
  signs: string | null;
  code?: string | null;
  confirmed: string | null;
  confirmedWhen: number;
  carried?: boolean;
  rotated?: boolean;
  host?: string | null;
  turnedAway: "disowned" | "unreadable" | "unconfirmed" | null;
}

export interface ThisMachine {
  id: string;
  name: string | null;
  os: string | null;
  code: string | null;
}

export const waitingMachines = (): Promise<Machine[]> => invoke("waiting_machines");
export const thisMachine = (): Promise<ThisMachine> => invoke("this_machine");
export const renameMachine = (name: string): Promise<ThisMachine> =>
  invoke("rename_machine", { name });
export const tidyMerged = (): Promise<number> => invoke("tidy_merged");
export type Repeated = { name: string; lists: number; tasks: number };
export const repeatedLists = (): Promise<Repeated[]> => invoke("repeated_lists");
export type Unvouched = { at: string; bytes: number };
export type Vouched = { kept: number; unlike: string[]; gone: string[] };
export interface Unvouching {
  found: Unvouched[];
  unread: string | null;
}

export const unvouchedAttachments = (): Promise<Unvouching> => invoke("unvouched_attachments");
export const vouchAttachments = (): Promise<Vouched> => invoke("vouch_attachments");

export interface Stray {
  file: string;
  title: string;
  bytes: number;
  when: number;
}

export interface Gone {
  id: string;
  file: string;
  title: string;
}

export interface Reviewed {
  tasks: number;
  lists: number;
  agrees: boolean;
  loose: number;
  looseBytes: number;
  astray: Astray[];
  came?: Record<string, Came>;
  unread: string | null;
  stranded: Stray[];
  missing: Gone[];
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
  unread: boolean;
  weight: number;
  syncs: boolean;
  shared: boolean;
  cloud?: string | null;
  backedUpAt: string | null;
  quiet: string[];
  shortcut: string | null;
  installed: "store" | "brew" | "download" | "disk";
  assistant: boolean;
}

export interface About {
  version: string;
  sandbox: string | null;
  repository: string;
  license: string;
  store: string;
  candidates: boolean;
  candidatesApply: boolean;
  keptByTheStore: boolean;
}

export type Holds = "everywhere" | "mine" | "shared";

export interface Freeing {
  gone: number;
  freed: number;
  done: boolean;
}

export const freeUp = (): Promise<Freeing> => invoke("free_up");
export const stopFreeing = (): Promise<void> => invoke("stop_freeing");

export interface Unused {
  files: number;
  bytes: number;
}

export const unusedAttachments = (days: number): Promise<Unused> =>
  invoke("unused_attachments", { days });
export const freeSpace = (days: number): Promise<Freeing> => invoke("free_space", { days });

export type Theme = "light" | "dark";

export interface Settings {
  quiet: string[];
  locale?: string;
  theme?: Theme;
  holds: Holds;
  shares: boolean;
  onlySharedAbove: number;
  onClose?: "hide" | "quit";
  quickKeys?: string;
}

export interface Logs {
  at: string;
  bytes: number;
  lines: string[];
}

export const logs = (most: number): Promise<Logs> => invoke("logs", { most });
export const noteTrouble = (code: string, name?: string): Promise<void> =>
  invoke("note_trouble", { code, name });

export type Route = "store" | "brew" | "download";

export interface Ready {
  version: string;
  route: Route;
  installs: boolean;
  coming?: boolean;
}

export interface Underway {
  stage: "getting" | "installing";
  far: number;
}

export const updateReady = (nowPlease?: boolean): Promise<Ready | null> =>
  invoke("update_ready", { nowPlease });

export const updateInstall = (): Promise<void> => invoke("update_install");

export const noteBreak = (kind: string, said: string | null, frames: string): Promise<void> =>
  invoke("note_break", { kind, said, frames });
export const settings = (): Promise<Settings> => invoke("settings");
export const keepSettings = (settings: Settings): Promise<Settings> =>
  invoke("keep_settings", { settings });

export const about = (): Promise<About> => invoke("about");

export const updateCandidates = (wants: boolean): Promise<void> =>
  invoke("update_candidates", { wants });
export const notices = (): Promise<string> => invoke("notices");
export const licences = (): Promise<string> => invoke("licences");

export const starDue = (): Promise<boolean> => invoke("star_due");

export const starDone = (): Promise<void> => invoke("star_done");

export interface DoorDue {
  why: "installed" | "mute" | "knocked";
  by: string | null;
}

export const doorDue = (): Promise<DoorDue | null> => invoke("door_due");

export const doorDone = (): Promise<void> => invoke("door_done");

export const doorHeard = (): Promise<void> => invoke("door_heard");
export const facts = (names: boolean, paths: boolean): Promise<Facts> =>
  invoke("facts", { names, paths });
export const keepReport = (at: string, text: string, logs: boolean): Promise<void> =>
  invoke("keep_report", { at, text, logs });
export const rebuild = (): Promise<void> => invoke("rebuild");
export const checked = (): Promise<Reviewed> => invoke("checked");
export const twinned = (): Promise<Pair[]> => invoke("twinned");
export const twinsJoin = (keep: string, go: string[]): Promise<Joined> =>
  invoke("twins_join", { keep, go });
export const syncState = (): Promise<Carrying> =>
  invoke<Carrying>("sync_state").then((state) => {
    speakOf(state?.cloudProvider);
    return state;
  });
export const chooseSync = (dest?: string): Promise<void> => invoke("choose_sync", { dest });

export interface Connected {
  provider: string;
  account: string;
  email: string;
}

export interface LastRound {
  at: number;
  code: string;
  name?: string | null;
  left: number;
  names: string[];
  cause?: Cause | null;
  rounds?: number;
  since?: number;
}

/** What stood between this computer and the cloud when a round could not reach it. */
export type Cause = "blocked" | "filtered" | "slow";

export interface ReadSoFar {
  done: number;
  whole: number;
  at: number;
}

export interface CloudStatus {
  provider: string;
  account: string;
  email: string;
  joined: boolean;
  unitsSpent: number;
  unitsADay: number;
  bytesSpent: number;
  bytesADay: number;
  pausedHere: boolean;
  haltedUntil?: number | null;
  owed: number;
  last?: LastRound | null;
  lost?: boolean;
  adopts?: boolean;
  running?: boolean;
  sealingElsewhere?: "on" | "off" | null;
  reading?: ReadSoFar | null;
}

export const cloudConnect = (provider: string): Promise<Connected> =>
  invoke("cloud_connect", { provider });
export const cloudConnectCancel = (): Promise<void> => invoke("cloud_connect_cancel");
export interface DavSignIn {
  address?: string;
  user?: string;
  account?: string;
  password: string;
}
export const cloudConnectDav = (signIn: DavSignIn): Promise<Connected> =>
  invoke("cloud_connect_dav", { ...signIn });
export const cloudChoose = (account: string, email: string, provider?: string): Promise<void> =>
  invoke<void>("cloud_choose", { account, email, provider }).then(() => speakOf(provider));
export const cloudForget = (account?: string, provider?: string): Promise<void> =>
  invoke("cloud_forget", { account, provider });
export const cloudRevoke = (account?: string, provider?: string): Promise<void> =>
  invoke("cloud_revoke", { account, provider });
export const cloudPause = (paused: boolean): Promise<void> => invoke("cloud_pause", { paused });
export const cloudStatus = (): Promise<CloudStatus | null> =>
  invoke<CloudStatus | null>("cloud_status").then((status) => {
    if (status) speakOf(status.provider);
    return status;
  });

export type Health = "reachable" | "inspected" | "blocked" | "slow" | "troubled";

export const cloudHealth = (provider: string): Promise<Health> =>
  invoke("cloud_health", { provider });

export type Toward =
  | { kind: "alone" }
  | { kind: "folder"; at: string }
  | { kind: "cloud"; account: string; email: string; provider?: string };
export interface Weight {
  files: number;
  bytes: number;
}
export interface MovePlan {
  down: Weight;
  up: Weight;
  fromCloud: boolean;
  toCloud: boolean;
  others: number;
  free: number | null;
  short: number;
}
export type Moving =
  | {
      state: "done";
      told: boolean;
      leftCloud?: { provider: string; account: string; email: string } | null;
    }
  | {
      state: "waiting";
      stage: "homing" | "filling";
      left: Weight;
      reason?: Waiting["reason"] | "unsettled" | "fetching" | null;
      wait?: number | null;
      cause?: Cause | null;
    };
export const movePlan = (toward: Toward): Promise<MovePlan> => invoke("move_plan", { toward });
export const moveTo = (toward: Toward): Promise<Moving> => invoke("move_to", { toward });
export const movePending = (): Promise<Toward | null> => invoke("move_pending");
export const moveDropped = (): Promise<void> => invoke("move_dropped");
export const cloudWipe = (account: string, email: string, provider?: string): Promise<number> =>
  invoke("cloud_wipe", { account, email, provider });
export const cloudStranded = (): Promise<Weight> => invoke("cloud_stranded");
export const cloudAbandon = (): Promise<void> => invoke("cloud_abandon");
export const placeRaised = (): Promise<string | null> => invoke("place_raised");
export const placeRaisedTold = (): Promise<void> => invoke("place_raised_told");
export type Carried = "came" | "sent" | "both" | "same" | "busy";

export interface Settled {
  carried: Carried;
  /** Seconds the round that turned this one away has run. */
  running?: number;
  undecided: string[];
  unreadable: string[];
  disowned: string[];
  unconfirmed: string[];
  waiting: string[];
  astray: string[];
  unprojected: boolean;
  joined: string[];
  deferred?: Waiting | null;
  refused?: { code: string; name?: string | null } | null;
  adopted?: boolean;
  owed?: boolean;
}

export type Waiting = {
  reason: "budget" | "limit" | "offline" | "busy" | "changed" | "paused" | "troubled" | "unsent";
  left: number;
  names: string[];
  cause?: Cause | null;
  rounds?: number;
  since?: number;
};

export const whatWentAmiss = (said: Settled) =>
  (said.unprojected && "ownLogUnreadable") ||
  (said.astray?.length && "someDocsAstray") ||
  (said.disowned?.length && "someoneDisowned") ||
  (said.waiting?.length && "someoneUnconfirmedHolds") ||
  (said.unconfirmed?.length && "someoneUnconfirmed") ||
  (said.unreadable?.length && "someoneUnreadable") ||
  null;

const WAITS = {
  budget: "deferredBudget",
  limit: "deferredLimit",
  offline: "deferredOffline",
  paused: "deferredPaused",
  troubled: "deferredTroubled",
  unsent: "deferredUnsent",
} as const;

// A round left for later is no error, and a busy or changed one retries by itself, so only these are said.
export const whatWaits = (said: Settled): (typeof WAITS)[keyof typeof WAITS] | null => {
  const reason = said.deferred?.reason;
  return reason && reason in WAITS ? WAITS[reason as keyof typeof WAITS] : null;
};

export const syncNow = (way?: "push" | "pull" | "again"): Promise<Settled> =>
  invoke("sync_now", { way });
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
  binary?: string;
}

export const reachable = (): Promise<Reach> => invoke("reachable");

export interface Wired {
  id: string;
  name: string;
  also?: string | null;
  place?: string | null;
  at: string;
  wired: boolean;
  astray: boolean;
  gone?: boolean;
  points?: string | null;
  maybe: boolean;
  byHand: boolean;
  manual: { file: string; snippet: string; cli?: string | null };
}

export interface InDistro {
  distro: { name: string; shut: boolean; command?: string | null };
  seen: Wired[];
}

export const seenAgents = (): Promise<Wired[]> => invoke("wiring");
export const wiringKnown = (): Promise<Wired[]> => invoke("wiring_known");
export const wslDistros = (): Promise<string[]> => invoke("wsl_distros");
export const wslSeen = (name: string): Promise<InDistro> => invoke("wsl_seen", { name });

/** One assistant as the person meets it: what it is called, whether it is wired, what it wrote. */
export interface Assistant {
  via: string | null;
  named: string;
  wired: boolean | null;
  filed: number;
  wrote: number;
  last: string | null;
}
export const assistants = (): Promise<Assistant[]> => invoke("assistants");
export const wireAgent = (id: string): Promise<Wired[]> => invoke("wire", { id });
export const unwireAgent = (id: string): Promise<Wired[]> => invoke("unwire", { id });

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
export const shortcutsOffered = (): Promise<string[]> => invoke("shortcuts_offered");
export const keepShortcut = (keys: string): Promise<string | null> =>
  invoke("keep_shortcut", { keys });

export const closeWindow = (how?: "hide" | "quit", remember?: boolean): Promise<void> =>
  invoke("close_window", { how, remember });
export const keepLocale = (locale?: string): Promise<string | null> =>
  invoke("keep_locale", { locale });
export const keepTheme = (theme?: Theme): Promise<Theme | null> => invoke("keep_theme", { theme });
export const keepClosing = (how: "hide" | "quit" | "ask"): Promise<void> =>
  invoke("keep_closing", { how });
export const guide = (own?: boolean): Promise<Doc> => invoke("guide", { own });
export type BackedUp = { bytes: number; leftOut: number; leftOutBytes: number };
export const backUp = (into: string): Promise<BackedUp> => invoke("back_up", { into });
export const retireAttachment = (reference: string): Promise<void> =>
  invoke("retire_attachment", { reference });
export const retireAttachments = (references: string[]): Promise<number> =>
  invoke("retire_attachments", { references });
export const removeMachine = (id: string): Promise<void> => invoke("remove_machine", { id });
export const confirmMachineKey = (id: string, key: string): Promise<void> =>
  invoke("confirm_machine_key", { id, key });
export const joinThem = (into: string): Promise<number> => invoke("join_them", { into });

export const takeOver = (into: string): Promise<number> => invoke("take_over", { into });

export const mergeStores = (into: string): Promise<boolean> => invoke("merge_stores", { into });

export type Kin = "sameLineage" | "clash" | "unsure" | "strangers";

export const syncKin = (): Promise<Kin> => invoke("sync_kin");

export interface Joining {
  kin: Kin;
  fresh: boolean;
  holds: boolean;
  alias: string | null;
  coming?: boolean;
}

export const joining = (): Promise<Joining> => invoke("joining");

/** Whether a place already holds a store, in one look: `fresh` is this machine never having written. */
export interface Peeked {
  fresh: boolean;
  /** Null when the place could not be asked. */
  holds: boolean | null;
}

export const placePeek = (toward?: Toward): Promise<Peeked> =>
  invoke("place_peek", { toward: toward ?? null });

export const folderAstir = (): Promise<string> => invoke("folder_astir");
export const restore = (from: string): Promise<number> => invoke("restore", { from });

export const revealed = (path: string): Promise<void> => invoke("revealed", { path });
export const stillAtOdds = (ids: string[]): Promise<string[]> => invoke("still_at_odds", { ids });
export const leaveHere = (anyway?: boolean): Promise<string[]> => invoke("leave_here", { anyway });

export const copied = (text: string): Promise<void> => writeText(text);

export const weighs = (reference: string): Promise<number> => invoke("weighs", { reference });

export const roomy = (): Promise<number> => invoke("roomy");
export const reopen = (id: string): Promise<Task> => invoke("reopen", { id });
// The last words typed on its page have to land before letting it go puts the page away.
export const discard = async (id: string): Promise<Task> => {
  await settled();
  return invoke("discard", { id });
};
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
  /// The folder's own mark: only the one that was shelved offers to come back.
  archived: boolean;
  /// What the archive holds, the folders above it counted in.
  away: boolean;
}

export type Paper = "a4" | "letter" | "folio" | "tabloid";

export interface Marked {
  at: string;
  said: string;
  via?: string | null;
  reason?: string | null;
  instead?: string | null;
}

export interface Compaction {
  at: string;
  via?: string | null;
}

export interface Filed {
  id: string;
  file: string;
  title: string;
  told?: boolean;
  bytes?: number | null;
  wrote?: string | null;
  folder: string | null;
  pageOf?: string | null;
  /// The document's own mark, which is what its own menu can answer for.
  archived: boolean;
  /// What the archive holds, the folder above it counted in.
  away: boolean;
  locked?: boolean;
  gone?: boolean;
  guest?: string | null;
  tags?: string[];
  flagged?: Marked | null;
  compacted?: Compaction | null;
  folderWas?: string[] | null;
}

export interface Papers {
  folders: Folded[];
  docs: Filed[];
}

export interface DocFacts {
  made: number | null;
  wrote: number | null;
  bytes: number;
  pages: number;
  author: string | null;
  editor: string | null;
  born: string | null;
  signedAs: string | null;
  editedAs: string | null;
}

export const ALIAS_AT_MOST = 40;
export const NAME_AT_MOST = 80;

export interface Signed {
  alias: string | null;
  name: string | null;
  before: string[];
  mine: number;
}

export const signed = (): Promise<Signed> => invoke("signed");
export const sign = (alias?: string): Promise<Signed> => invoke("sign", { alias });
export const signName = (name?: string): Promise<Signed> => invoke("sign_name", { name });
export const signTheRest = (): Promise<number> => invoke("sign_the_rest");

export const docFacts = (id: string): Promise<DocFacts> => invoke("doc_facts", { id });

export const docs = (): Promise<Papers> => invoke("docs");
export const docsCatchUp = (): Promise<Filed[]> => invoke("docs_catch_up");
export const folderAdd = (
  name: string,
  parent?: string,
  icon?: string,
  color?: string,
): Promise<string> => invoke("folder_add", { name, parent, icon, color });
export const folderRename = (id: string, name: string): Promise<void> =>
  invoke("folder_rename", { id, name });
export const folderLook = (id: string, icon?: string, color?: string): Promise<void> =>
  invoke("folder_look", { id, icon, color });
export const folderDrop = (id: string): Promise<void> => invoke("folder_drop", { id });
export const docFile = (id: string, folder?: string, before?: string): Promise<void> =>
  invoke("doc_file", { id, folder, before });
export const docRead = (id: string): Promise<string> => invoke("doc_read", { id });
export const docDrawn = (body: string): Promise<string> =>
  invoke<ArrayBuffer>("doc_drawn", { body }).then((bytes) => new TextDecoder().decode(bytes));
export const docPlain = (body: string): Promise<string> => invoke("doc_plain", { body });
export const noteDrawn = (body: string): Promise<string> => invoke("note_drawn", { body });

export type Piece = { readonly said: string; readonly fixed: string };

export type Mend = {
  readonly from: number;
  readonly to: number;
  readonly fixed: string;
  readonly pieces: readonly Piece[];
};

export const mendsIn = (body: string): Promise<Mend[]> => invoke("mends_in", { body });
export interface Opened {
  body: string;
  print: string;
}
export const docOpened = (id: string): Promise<Opened> => invoke("doc_opened", { id });
export const docWrite = (
  id: string,
  body: string,
  anyway?: boolean,
  since?: string,
): Promise<Doc & { print: string }> => invoke("doc_write", { id, body, anyway, since });
export const docMerge = (
  id: string,
  base: string,
  mine: string,
): Promise<(Opened & { title: string }) | null> => invoke("doc_merge", { id, base, mine });
export const docBack = (id: string): Promise<Doc> => invoke("doc_back", { id });
export const docBackable = (id: string): Promise<boolean> => invoke("doc_backable", { id });
export const folderFile = (id: string, parent?: string, before?: string): Promise<void> =>
  invoke("folder_file", { id, parent, before });

export const keepFile = (at: string, bytes: number[]): Promise<void> =>
  invoke("keep_file", { at, bytes });
export const pagePdf = (html: string, paper: string, secret?: boolean): Promise<number[]> =>
  invoke("page_pdf", { html, paper, secret });

export interface Identity {
  held: boolean;
  elsewhere: boolean;
  fingerprint: string | null;
  suffix: string | null;
  confirmed: boolean;
  alias: string | null;
  machines: { id: string; name: string; here: boolean; ours: boolean }[];
}

export const identityShown = (): Promise<Identity> => invoke("identity_shown");
export const identityPhrase = (): Promise<string> => invoke("identity_phrase");
export const identityConfirm = (said: [number, string][]): Promise<number[]> =>
  invoke("identity_confirm", { said });
export const identityBegin = (): Promise<string> => invoke("identity_begin");

export interface SealTold {
  on: boolean;
  step: "claimed" | "copying" | "closed" | "settling" | "done";
  done: number;
  left: number;
  refused: number;
  stopped: boolean;
  why: string | null;
  waiting?: Cause | "offline" | "cloud" | "local" | "busy" | "budget" | "paused" | null;
}

export interface SealShown {
  cloud: string | null;
  state: "none" | "plain" | "sealed" | "locked" | "turning" | "elsewhere";
  joined: boolean;
  held: boolean;
  confirmed: boolean;
  phraseElsewhere: boolean;
  behind: string[];
  turning: SealTold | null;
  wiping?: boolean;
  orphan: boolean;
  plainLeft?: boolean;
  reading?: ReadSoFar | null;
}

export interface SealRoom {
  free: number | null;
  needed: number;
}

export const sealingShown = (): Promise<SealShown> => invoke("sealing_shown");
export const sealingRoom = (): Promise<SealRoom> => invoke("sealing_room");
export const sealingBegin = (on: boolean): Promise<void> => invoke("sealing_begin", { on });
export const sealingCancel = (): Promise<void> => invoke("sealing_cancel");
export const sealingWipe = (): Promise<number> => invoke("sealing_wipe");
export const sealingResume = (): Promise<void> => invoke("sealing_resume");

export interface Admitted {
  step: "idle" | "waiting" | "compare" | "trusted" | "done";
  code: string | null;
  by: string | null;
}

export interface Newcomer {
  device: string;
  name: string;
  at: number;
  taken: boolean;
}

export const admittingAsk = (): Promise<void> => invoke("admitting_ask");
export const admittingShown = (): Promise<Admitted> => invoke("admitting_shown");
export const admittingTrust = (code: string): Promise<void> => invoke("admitting_trust", { code });
export const admittingCancel = (): Promise<void> => invoke("admitting_cancel");
export const admittingLook = (): Promise<Newcomer[]> => invoke("admitting_look");
export const admittingOffer = (device: string): Promise<void> =>
  invoke("admitting_offer", { device });
export const admittingGrant = (device: string, code: string): Promise<void> =>
  invoke("admitting_grant", { device, code });
export const admittingRefuse = (device: string): Promise<void> =>
  invoke("admitting_refuse", { device });

export interface Heard {
  device: string;
  name: string;
  os: string | null;
  at: number;
}

export const sealingHeard = (): Promise<Heard[]> => invoke("sealing_heard");
export const sealingPhrase = (said: string): Promise<void> => invoke("sealing_phrase", { said });
export const docWord = (bodies: string[], at: string): Promise<void> =>
  invoke("doc_word", { bodies, at });

export const parted = (): Promise<void> => invoke("parted");
export const sow = (priority?: Priority): Promise<void> => invoke("sow", { priority });

export const docAway = (id: string, away: boolean): Promise<void> =>
  invoke("doc_away", { id, away });
export const folderAway = (id: string, away: boolean): Promise<void> =>
  invoke("folder_away", { id, away });
export const docLock = (id: string, shut: boolean): Promise<void> =>
  invoke("doc_lock", { id, shut });
export const docUnflag = (id: string): Promise<void> => invoke("doc_unflag", { id });
export const docKeepCompacted = (id: string): Promise<void> => invoke("doc_keep_compacted", { id });
export const docCompactedLast = (id: string): Promise<boolean> =>
  invoke("doc_compacted_last", { id });

export const docCopy = (id: string): Promise<Doc> => invoke("doc_copy", { id });

export interface Taken {
  files: number;
  missed: number;
  left: number;
}

export const docExport = (id: string, into: string): Promise<Taken> =>
  invoke("doc_export", { id, into });

export interface Afoot {
  stage: "packing" | "takingOut" | "landing";
  far: number;
  done: number;
  whole: number;
}

export interface Packed {
  docs: number;
  pages: number;
  folders: number;
  files: number;
  missed: number;
  left: number;
}

export interface Unpacked {
  docs: number;
  pages: number;
  folders: number;
  joined: number;
  files: number;
  missed: number;
}

export const spelled = (said: string): Promise<string> => invoke("spelled", { said });
export const docsPack = (which: string[], into: string, number?: string): Promise<Packed> =>
  invoke("docs_pack", { which, into, number });
export const docsUnpack = (from: string, number?: string): Promise<Unpacked> =>
  invoke("docs_unpack", { from, number });
export const docsTakeOut = (which: string[], into: string): Promise<Packed> =>
  invoke("docs_take_out", { which, into });
export const docFromHtml = (html: string): Promise<string> => invoke("doc_from_html", { html });

export const docFromText = (text: string): Promise<string | null> =>
  invoke("doc_from_text", { text });

export type Left = { missed: number; notes: number };

export type Arrived = Doc & Left;

export const docImport = (from: string, folder?: string): Promise<Arrived> =>
  invoke("doc_import", { from, folder });

export const docNew = (folder?: string, pageOf?: string): Promise<Doc> =>
  invoke("doc_new", { folder, pageOf });
export const docOrder = (id: string, body: string): Promise<boolean> =>
  invoke("doc_order", { id, body });
export const docPage = (id: string, pageOf?: string): Promise<void> =>
  invoke("doc_page", { id, pageOf });
export const docDrop = (id: string): Promise<void> => invoke("doc_drop", { id });

export const docAdopt = (file: string): Promise<Doc> => invoke("doc_adopt", { file });
export const docLetGo = (file: string): Promise<void> => invoke("doc_let_go", { file });

export const icons = (): Promise<string[]> => invoke("icons");
export const families = (): Promise<[string, number][]> => invoke("families");
export const listAdd = (name: string, icon?: string, color?: string): Promise<List> =>
  invoke("list_add", { name, icon, color });
export const listLook = (id: string, icon?: string, color?: string): Promise<List> =>
  invoke("list_look", { id, icon, color });
export const listRename = (id: string, name: string): Promise<List> =>
  invoke("list_rename", { id, name });
export const listDrop = (id: string): Promise<void> => invoke("list_drop", { id });
export const listArchive = (id: string): Promise<void> => invoke("list_archive", { id });
export const listUnarchive = (id: string): Promise<void> => invoke("list_unarchive", { id });
