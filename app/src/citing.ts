import { invoke } from "@tauri-apps/api/core";
import type { CardAsked } from "tisty-editor/read";
import type { Task } from "./core";

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

export type CardsSeen = {
  readonly here?: string;
  readonly acting: boolean;
  readonly more: boolean;
  readonly examples: boolean;
};

export const cardsDrawn = (asked: readonly CardAsked[], seen: CardsSeen): Promise<string[]> =>
  invoke("cards_drawn", { asked, seen });

export const taskStepMarked = (id: string, step: string, done: boolean): Promise<Task> =>
  invoke("task_step_marked", { id, step, done });
