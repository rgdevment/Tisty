import { invoke } from "@tauri-apps/api/core";

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
