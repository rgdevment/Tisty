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
  color?: string;
}

export const OPENED = "brainsty-opened";
export const OPEN_ASKED = "brainsty-open";
export const SPOKEN = "spoken";

export const brainNetwork = (near?: string): Promise<Network | null> =>
  invoke("brain_network", { near: near ?? null });

export const brainstyGroups = (): Promise<Group[]> => invoke("brainsty_groups");

export const showBrainsty = (near?: string): Promise<void> =>
  invoke("brainsty_show", { near: near ?? null });

export const openInMain = ({ kind, id, title }: Neuron): Promise<void> =>
  invoke("brainsty_open", { neuron: { kind, id, title } });
