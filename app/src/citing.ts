import { invoke } from "@tauri-apps/api/core";
import type { DateSpec, Status, Task } from "./core";

export interface Under {
  id: string;
  title: string;
}

export interface Citing {
  id: string;
  kind: "doc" | "page" | "task";
  title: string;
  state?: string;
  card: boolean;
  under?: Under[];
}

export const docCitedBy = (id: string): Promise<Citing[] | null> => invoke("doc_cited_by", { id });

export interface Heading {
  id: string;
  title: string;
  level: number;
}

export interface Glance {
  title: string;
  heading: string | null;
  html: string;
  more: boolean;
}

export const docHeads = (id: string): Promise<Heading[]> => invoke("doc_heads", { id });

export const docGlance = (id: string, at?: string): Promise<Glance> =>
  invoke("doc_glance", { id, at });

export interface CitedStep {
  id: string;
  html: string;
  done: boolean;
}

export interface Cited {
  id: string;
  title: string;
  status: Status;
  list?: string;
  date?: DateSpec;
  deadline?: DateSpec;
  completed_at?: string;
  steps: CitedStep[];
  parts?: { closed: number; all: number };
  part_of?: string;
}

export const taskCards = (ids: string[]): Promise<Cited[]> => invoke("task_cards", { ids });

export const taskStepMarked = (id: string, step: string, done: boolean): Promise<Task> =>
  invoke("task_step_marked", { id, step, done });
