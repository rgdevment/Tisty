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
