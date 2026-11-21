import { invoke } from "@tauri-apps/api/core";
import type { Filed, Folded } from "./core";
import { t } from "./locales";

export const AGENTS_ICON = "robot";
export const AGENTS_MORE = "tisty:agents-more";

export const bringMoreAgents = () => window.dispatchEvent(new Event(AGENTS_MORE));
export const AGENTS_RESCAN = "tisty:agents-rescan";
/// «Look again»: the assistants are found anew and only what is new comes ticked.
export const rescanAgents = () => window.dispatchEvent(new Event(AGENTS_RESCAN));
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
  syncs: boolean;
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
  held: number;
  consented: boolean;
  allOn: boolean;
  assistants: { id: string; name: string; on: boolean }[];
}

export const agentsState = (): Promise<AgentsState> => invoke("agents_state");

/// Syncing needs the folder, this computer's yes and its switch on.
export const agentsLit = (state: AgentsState): boolean =>
  !!state.folder && state.consented && state.allOn;

/// Only a folder this computer already allowed syncs at once; the rest goes through the first time.
export const onlySwitchedOff = (state: AgentsState): boolean =>
  !!state.folder && state.consented && !state.allOn;

export type AgentsStage = "unbrought" | "brought" | "syncs";

// The count comes from the core alone, so the sidebar and Settings never tell two stages apart.
export const agentsStage = (root: string | null, state: AgentsState | null): AgentsStage | null => {
  if (!root) return "unbrought";
  if (!state) return null;
  if (agentsLit(state)) return "syncs";
  return state.held > 0 || state.consented ? "brought" : "unbrought";
};

/// The coral is Agents' own, and only while it syncs: the purple belongs to Diagrams.
export const agentsPaint = (stage: AgentsStage | null): string =>
  stage === "syncs" ? "text-hue-coral" : "text-faint";

export const isAgentsFolder = (one: Pick<Folded, "parent" | "icon" | "name">): boolean =>
  !one.parent && one.icon === AGENTS_ICON && one.name === AGENTS_NAME;

/// The folder keeps its stored name, which older builds look for; the window says it in the person's language.
export const folderShown = (one: Pick<Folded, "parent" | "icon" | "name">): string =>
  isAgentsFolder(one) ? t("agentsName") : one.name;

export const AGENTS_CHANGED = "tisty:agents-changed";
export const AGENTS_FIRST = "tisty:agents-first";
export const askAgents = () => window.dispatchEvent(new Event(AGENTS_FIRST));
export const AGENTS_SYNC = "tisty:agents-sync";
/// «Sync», from the folder's line, its menu or the settings card: at once where this computer said yes, else the first time.
export const syncAgents = () => window.dispatchEvent(new Event(AGENTS_SYNC));

export const agentsLook = (): Promise<AgentsLook> => invoke("agents_look");
export const agentsRescan = (): Promise<AgentsLook> => invoke("agents_rescan");
export const agentsRows = (only?: string[]): Promise<AgentRow[]> =>
  invoke("agents_rows", { only: only ?? null });
/// `sync` is null to keep the switch as it is, as «Bring another file» does.
export const agentsBring = (chosen: string[], sync: boolean | null): Promise<Brought> =>
  invoke("agents_bring", { chosen, sync });
export const agentsKeep = (at: string, tisty: boolean): Promise<Rounded> =>
  invoke("agents_keep", { at, tisty });
export const agentsTurn = (assistant: string | null, on: boolean): Promise<Rounded> =>
  invoke("agents_turn", { assistant, on });
export const agentsRound = (): Promise<Rounded> => invoke("agents_round");
export const agentsEmpty = (): Promise<boolean> => invoke("agents_empty");
export const agentsLoose = (at: string, away: boolean): Promise<Rounded> =>
  invoke("agents_loose", { at, away });
export const agentsLead = (): Promise<Rounded> => invoke("agents_lead");

export const agentsRoots = (folders: readonly Folded[]): Folded[] =>
  folders
    .filter((one) => one.parent === null && !one.archived && isAgentsFolder(one))
    .sort((a, b) => a.id.localeCompare(b.id));

export const agentsRoot = (folders: readonly Folded[]): Folded | undefined =>
  agentsRoots(folders)[0];

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
