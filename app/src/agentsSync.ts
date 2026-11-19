import { invoke } from "@tauri-apps/api/core";
import type { Filed, Folded } from "./core";

export const AGENTS_ICON = "robot";
export const AGENTS_MORE = "tisty:agents-more";

export const bringMoreAgents = () => window.dispatchEvent(new Event(AGENTS_MORE));
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

export interface Loose {
  at: string;
  assistant: string;
  changed: boolean;
}

export interface AgentsLook {
  folder: string | null;
  docs: number;
  changes: number;
  waiting: number;
  unagreed: number;
  proposed: number;
  consented: boolean;
  allOn: boolean;
  asks: boolean;
  assistants: AssistantSeen[];
  both: Both[];
  loose: Loose[];
  unled: string[];
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
  loose: Loose[];
  unled: string[];
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

export interface AgentRow {
  file: string;
  kind: "rule" | "skill" | "memory";
  level: string;
  name: string;
  label: string;
  skill: string | null;
}

export interface AgentsState {
  folder: string | null;
  consented: boolean;
  allOn: boolean;
  assistants: { id: string; name: string }[];
}

export const agentsState = (): Promise<AgentsState> => invoke("agents_state");

/// Off while there is no folder, this computer never said yes, or its sync is switched off.
export const agentsLit = (state: AgentsState): boolean =>
  !!state.folder && state.consented && state.allOn;

/// Only a folder this computer already allowed switches back on; the rest goes through the first time.
export const onlySwitchedOff = (state: AgentsState): boolean =>
  !!state.folder && state.consented && !state.allOn;

export const AGENTS_CHANGED = "tisty:agents-changed";
export const AGENTS_ASLEEP = "tisty:agents-asleep";
export const AGENTS_LIGHT = "tisty:agents-light";
/// The short notice an AGENTS that is off answers with: what was found, and turn on, not now or hide.
export const askToLight = () => window.dispatchEvent(new Event(AGENTS_ASLEEP));
/// Turning on straight away, as the notice's «Turn on» and the settings card do.
export const lightAgents = () => window.dispatchEvent(new Event(AGENTS_LIGHT));

export const agentsLook = (): Promise<AgentsLook> => invoke("agents_look");
export const agentsRows = (only?: string[]): Promise<AgentRow[]> =>
  invoke("agents_rows", { only: only ?? null });
export const agentsBring = (chosen: string[]): Promise<Brought> =>
  invoke("agents_bring", { chosen });
export const agentsKeep = (at: string, tisty: boolean): Promise<Rounded> =>
  invoke("agents_keep", { at, tisty });
export const agentsTurn = (assistant: string | null, on: boolean): Promise<Rounded> =>
  invoke("agents_turn", { assistant, on });
export const agentsRound = (): Promise<Rounded> => invoke("agents_round");
export const agentsEmpty = (): Promise<boolean> => invoke("agents_empty");
export const agentsLoose = (at: string, away: boolean): Promise<Rounded> =>
  invoke("agents_loose", { at, away });
export const agentsLead = (): Promise<Rounded> => invoke("agents_lead");

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
