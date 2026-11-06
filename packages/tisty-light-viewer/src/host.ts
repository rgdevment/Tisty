import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type Head = { readonly level: number; readonly said: string; readonly id: string };

export type Refused = { readonly code: string; readonly name?: string };

export type Opened =
  | {
      readonly kind: "drawn";
      readonly name: string;
      readonly place: string;
      readonly path: string;
      readonly first: string;
      readonly more: boolean;
      readonly heads: readonly Head[];
    }
  | {
      readonly kind: "missed";
      readonly name: string;
      readonly place: string;
      readonly path: string;
      readonly refusal: Refused;
    };

export type Facts = { readonly wrote: number | null; readonly bytes: number };

export type Left = { readonly missed: number; readonly notes: number };

const here = getCurrentWindow();

export const opened = (): Promise<Opened> => invoke("view_read");
export const theRest = (): Promise<string[]> => invoke("view_rest");
export const source = (): Promise<string> => invoke("view_source");
export const facts = (): Promise<Facts> => invoke("view_facts");
export const editCopy = (): Promise<Left> => invoke("view_import");
export const openLink = (url: string): Promise<void> => invoke("open_link", { url });

export const pictureNear = (src: string): Promise<string> => invoke("view_picture", { src });
export const pictureFar = (src: string): Promise<string> => invoke("view_picture_far", { src });
export const follow = (to: string): Promise<void> => invoke("view_follow", { to });

export const reveal = (path: string): Promise<void> =>
  import("@tauri-apps/plugin-opener").then(({ revealItemInDir }) => revealItemInDir(path));

export const shown = (): Promise<void> => here.show();
export const minimized = (): Promise<void> => here.minimize();
export const maximized = (): Promise<void> => here.toggleMaximize();
export const closed = (): Promise<void> => here.close();
