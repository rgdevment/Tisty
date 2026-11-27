import { invoke } from "@tauri-apps/api/core";
import { useSyncExternalStore } from "react";
import { inAgents } from "./agentsSync";
import type { Filed, Papers } from "./core";

export type Place = "plain" | "ideas" | "diary" | "agents";
export type Making = "doc" | "diagram" | "folder" | "import" | "parcel";

export interface Rules {
  makes: Making[];
  pages: boolean;
  moves: boolean;
  copies: boolean;
  goes: boolean;
}

export type Ruling = Record<Place, Rules>;

const OPEN: Rules = {
  makes: ["doc", "diagram", "folder", "import", "parcel"],
  pages: true,
  moves: true,
  copies: true,
  goes: true,
};

const told = new Set<() => void>();
let ruling: Ruling | null = null;
let asking: Promise<void> | null = null;

// The core says what each place takes; until it answers every place offers all, and the core still refuses.
export const placesRules = (): Promise<void> => {
  asking ??= invoke<Ruling>("places_rules")
    .then((said) => {
      ruling = said;
      for (const one of told) one();
    })
    .catch(() => {
      asking = null;
    });
  return asking;
};

export const rulesOf = (place: Place): Rules => ruling?.[place] ?? OPEN;

export const useRuling = (): Ruling | null =>
  useSyncExternalStore(
    (change) => {
      told.add(change);
      void placesRules();
      return () => told.delete(change);
    },
    () => ruling,
  );

export const placeOf = (doc: Filed, papers: Papers, board?: string | null): Place => {
  const book = (doc.pageOf && papers.docs.find((one) => one.id === doc.pageOf)) || doc;
  if (board && book.file === board) return "ideas";
  if (doc.diary || book.diary) return "diary";
  return inAgents(book.folder, papers.folders) ? "agents" : "plain";
};

export const folderPlace = (folder: string | null | undefined, papers: Papers): Place =>
  inAgents(folder, papers.folders) ? "agents" : "plain";

/// `cards` is false in an agent's own file, which its agent reads as it stands.
export interface Placed {
  place: Place;
  cards: boolean;
}

export const docPlace = (file: string): Promise<Placed> => invoke<Placed>("doc_place", { file });
