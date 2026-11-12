import { invoke } from "@tauri-apps/api/core";

export interface Jotted {
  file: string;
  made: boolean;
}

export const jot = (idea: string, title: string): Promise<Jotted> => invoke("jot", { idea, title });

export const brainstormOf = (): Promise<string | null> => invoke("brainstorm_of");
