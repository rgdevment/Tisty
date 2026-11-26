import { invoke } from "@tauri-apps/api/core";
import type { Folded } from "./core";
import { fill } from "./locales";
import { sure } from "./sure";

export type Contents = "up" | "into" | "drop";

export interface FolderAnswer {
  contents: Contents;
  into?: string;
}

export interface FolderAsked {
  folder: Folded;
  /** The folder above it, where «up» hands what it holds; none is the top of the library. */
  up: Folded | null;
  docs: number;
  folders: number;
  /** Inside Agents what it holds is only handed up: deleting it goes through Agents' own question. */
  handsOnly: boolean;
}

type Asking = (asked: FolderAsked) => Promise<FolderAnswer | null>;

let asking: Asking | null = null;

export const asksFolderDrop = (one: Asking | null): void => {
  asking = one;
};

// With no window to ask in, the plain question hands what it holds up a level.
export const folderDropAsked = (asked: FolderAsked): Promise<FolderAnswer | null> =>
  asking
    ? asking(asked)
    : sure(fill("dropFolderSure", asked.folder.name)).then((yes) =>
        yes ? { contents: "up" } : null,
      );

export const folderDropped = (id: string, answer: FolderAnswer): Promise<void> =>
  invoke("folder_drop", { id, contents: answer.contents, into: answer.into });
