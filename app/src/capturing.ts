import { invoke } from "@tauri-apps/api/core";
import type { Counted, DateSpec, List, Priority, Repeat, Task, View } from "./core";

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

export type Born = { doc?: string; before?: string; whole?: string };
export const captureKnown = (): Promise<{ lists: List[]; tags: Counted[] }> =>
  invoke("capture_known");
export const capture = (text: string, view?: View, edits?: Edits, born?: Born): Promise<Task> =>
  invoke("capture", { text, locale: navigator.language, view, edits, born });
