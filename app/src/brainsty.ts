import { invoke } from "@tauri-apps/api/core";

// The core sends each line's kind as its place in this list.
export const TIES = ["cite", "mention", "structure", "tag", "uses"] as const;

export type Tie = (typeof TIES)[number];

export type NeuronKind = "doc" | "page" | "task" | "tag" | "file";

export interface Neuron {
  id: string;
  kind: NeuronKind;
  title: string;
  group?: string;
  state?: string;
  when?: string;
}

export interface LeftOut {
  nodes: number;
  edges: number;
}

// Both ends, the score in thousandths and the words both share: Tisty's guess, apart from the lines.
export type Guess = [number, number, number, string];

export interface Network {
  nodes: Neuron[];
  edges: [number, number, number][];
  near?: number;
  left_out?: LeftOut;
  alike?: Guess[];
}

export interface Group {
  id: string;
  name: string;
  color?: string | null;
  agents?: boolean;
}

export interface Near {
  id: string;
  kind: string;
  title: string;
  state?: string;
}

export type Due = "undated" | "ahead" | "today" | "late" | "underway" | "due_soon" | "overdue";

export interface Card {
  id: string;
  kind: string;
  title: string;
  state: string;
  standing?: Due;
  lang?: string;
  bytes?: number;
  words?: number;
  made?: string;
  wrote?: string;
  date?: string;
  deadline?: string;
  steps?: [number, number];
  steps_said?: number;
  source?: string;
  book?: Near;
  part_of?: Near;
  parts?: Near[];
  tags?: string[];
  cited_by?: Near[];
  mentioned_by?: Near[];
}

export const OPENED = "brainsty-opened";

export const brainNetwork = (near?: string): Promise<Network | null> =>
  invoke("brain_network", { near: near ?? null });

export const brainCard = (id: string): Promise<Card | null> => invoke("brain_card", { id });

export interface FileFacts {
  bytes: number;
  changed?: string;
}

export const brainFile = (id: string): Promise<FileFacts | null> => invoke("brain_file", { id });

export const taskAddress = (id: string): string => `tisty://task?id=${id}`;
