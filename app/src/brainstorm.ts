import { invoke } from "@tauri-apps/api/core";

export interface Jotted {
  file: string;
  made: boolean;
}

export interface Pinned {
  file: string;
  ideas: number;
  marked: number;
}

export const jot = (idea: string, title: string): Promise<Jotted> => invoke("jot", { idea, title });

export const brainstormOf = (): Promise<Pinned | null> => invoke("brainstorm_of");

export const ideaToDoc = (file: string, idea: string, from: string): Promise<void> =>
  invoke("idea_to_doc", { file, idea, from });

export const ideaUnflag = (file: string, heading: string, text: string): Promise<void> =>
  invoke("idea_unflag", { file, heading, text });
