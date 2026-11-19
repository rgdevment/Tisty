import { invoke } from "@tauri-apps/api/core";
import type { Filed, Folded } from "./core";

export const AGENTS_ICON = "robot";
export const AGENTS_NAME = "AGENTS";

export interface FileSeen {
  at: string;
  name: string;
  kind: "rule" | "skill" | "memory";
  skip: string | null;
  held: boolean;
}

export interface AssistantSeen {
  id: string;
  name: string;
  on: boolean;
  files: FileSeen[];
}

export interface AgentsLook {
  folder: string | null;
  docs: number;
  changes: number;
  waiting: number;
  unagreed: number;
  consented: boolean;
  allOn: boolean;
  asks: boolean;
  assistants: AssistantSeen[];
  both: Both[];
}

export interface Line {
  side: "same" | "tisty" | "file";
  text: string;
}

export interface Both {
  at: string;
  assistant: string;
  composed: boolean;
  lines: Line[];
}

export interface Told {
  at: string;
  assistant: string;
  files: string[];
  standing: string;
}

export interface Rounded {
  told: Told[];
  both: Both[];
}

export interface Refused {
  at: string;
  why: string;
  reason: string;
}

export interface Brought {
  made: number;
  refused: Refused[];
  round: Rounded;
}

export const agentsLook = (): Promise<AgentsLook> => invoke("agents_look");
export const agentsBring = (chosen: string[]): Promise<Brought> =>
  invoke("agents_bring", { chosen });
export const agentsKeep = (at: string, tisty: boolean): Promise<Rounded> =>
  invoke("agents_keep", { at, tisty });
export const agentsTurn = (assistant: string | null, on: boolean): Promise<Rounded> =>
  invoke("agents_turn", { assistant, on });
export const agentsRound = (): Promise<Rounded> => invoke("agents_round");
export const agentsEmpty = (): Promise<boolean> => invoke("agents_empty");

export const agentsRoot = (folders: readonly Folded[]): Folded | undefined =>
  folders
    .filter(
      (one) =>
        one.parent === null &&
        !one.archived &&
        one.icon === AGENTS_ICON &&
        one.name === AGENTS_NAME,
    )
    .sort((a, b) => a.id.localeCompare(b.id))[0];

export const agentsTree = (folders: readonly Folded[]): Set<string> => {
  const root = agentsRoot(folders);
  const tree = new Set<string>(root ? [root.id] : []);
  let grew = tree.size > 0;
  while (grew) {
    grew = false;
    for (const one of folders) {
      if (one.parent && tree.has(one.parent) && !tree.has(one.id)) {
        tree.add(one.id);
        grew = true;
      }
    }
  }
  return tree;
};

export const inAgents = (folder: string | null | undefined, folders: readonly Folded[]) =>
  !!folder && agentsTree(folders).has(folder);

export const agentsPrint = (docs: readonly Filed[], folders: readonly Folded[]): string => {
  const tree = agentsTree(folders);
  return docs
    .filter((one) => one.folder && tree.has(one.folder))
    .map((one) => `${one.file}:${one.wrote ?? ""}:${one.bytes ?? ""}:${one.away}`)
    .sort((a, b) => a.localeCompare(b))
    .join("|");
};
