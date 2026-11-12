import { invoke } from "@tauri-apps/api/core";

export interface Jotted {
  file: string;
  made: boolean;
}

export interface Pinned {
  file: string;
  ideas: number;
}

export const jot = (idea: string, title: string): Promise<Jotted> => invoke("jot", { idea, title });

export const brainstormOf = (): Promise<Pinned | null> => invoke("brainstorm_of");

export const ideaToDoc = (file: string, idea: string): Promise<void> =>
  invoke("idea_to_doc", { file, idea });
