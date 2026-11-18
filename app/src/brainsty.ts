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

export interface Network {
  nodes: Neuron[];
  edges: [number, number, number][];
  near?: number;
  left_out?: LeftOut;
}

export const OPENED = "brainsty-opened";

export const brainNetwork = (near?: string): Promise<Network | null> =>
  invoke("brain_network", { near: near ?? null });
