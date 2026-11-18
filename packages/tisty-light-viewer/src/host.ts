import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { CardAsked } from "tisty-sheet";

export type Head = { readonly level: number; readonly said: string; readonly id: string };

export type Refused = { readonly code: string; readonly name?: string };

export type Opened =
  | {
      readonly kind: "drawn";
      readonly name: string;
      readonly place: string;
      readonly path: string;
      readonly own: boolean;
      readonly first: string;
      readonly more: boolean;
      readonly heads: readonly Head[];
    }
  | {
      readonly kind: "missed";
      readonly name: string;
      readonly place: string;
      readonly path: string;
      readonly own: boolean;
      readonly refusal: Refused;
    };

export type Facts = { readonly wrote: number | null; readonly bytes: number };

export type Left = { readonly missed: number; readonly notes: number };

const here = getCurrentWindow();

export const opened = (): Promise<Opened> => invoke("view_read");
export const theRest = (): Promise<string[]> => invoke("view_rest");
export const source = (): Promise<string> => invoke("view_source");
export const plain = (): Promise<string> => invoke("view_plain");
export const address = (): Promise<string> => invoke("view_address");
export const copied = (text: string): Promise<void> =>
  invoke("plugin:clipboard-manager|write_text", { text });
export const facts = (): Promise<Facts> => invoke("view_facts");
export const editCopy = (): Promise<Left | null> => invoke("view_import");
export const editAsked = (act: () => void): Promise<() => void> => here.listen("view-edit", act);
export const openLink = (url: string): Promise<void> => invoke("open_link", { url });

export const pictureNear = (src: string): Promise<string> => invoke("view_picture", { src });
export const pictureFar = (src: string): Promise<string> => invoke("view_picture_far", { src });
export const follow = (to: string): Promise<void> => invoke("view_follow", { to });

export const cardsOf = (asked: readonly CardAsked[]): Promise<string[]> =>
  invoke("view_cards", { asked });
export const showDoc = (file: string): Promise<void> => invoke("view_go", { file });
export const showTask = (task: string): Promise<void> => invoke("view_task", { task });
export const openAttached = (reference: string): Promise<void> => invoke("opened", { reference });
export const served = (src: string): Promise<string> => invoke("view_served", { src });
export const keptHere = (src: string): Promise<boolean> => invoke("view_kept_here", { src });
export const weighs = (src: string): Promise<number> => invoke("view_weighs", { src });
export const fileUrl = (real: string): string => convertFileSrc(real);
export const drawsHeic = (): Promise<boolean> => invoke("draws_heic");

export const reveal = (path: string): Promise<void> =>
  import("@tauri-apps/plugin-opener").then(({ revealItemInDir }) => revealItemInDir(path));

// Opened from another app, Tisty is not in front, and a window only shown would stay behind it.
export const shown = (): Promise<void> => invoke("view_fronted");
export const themed = (dark: boolean): Promise<void> => here.setTheme(dark ? "dark" : "light");
export const closed = (): Promise<void> => here.close();
