import { invoke } from "@tauri-apps/api/core";

export interface HelpEntry {
  slug: string;
  title: string;
  section: string;
  order: number;
  twin: string;
}

export interface HelpPictures {
  weighs: number;
  missing: number;
  kept: Record<string, string>;
}

export const helpPages = (spanish: boolean): Promise<HelpEntry[]> =>
  invoke("help_pages", { spanish });
export const helpPage = (spanish: boolean, slug: string): Promise<string | null> =>
  invoke("help_page", { spanish, slug });
export const helpPictures = (spanish: boolean): Promise<HelpPictures> =>
  invoke("help_pictures", { spanish });
export const helpPicturesFetch = (spanish: boolean): Promise<HelpPictures> =>
  invoke("help_pictures_fetch", { spanish });

export const helpReference = (spanish: boolean): Promise<string> =>
  invoke("help_reference", { spanish });
export const helpDrawn = (body: string): Promise<string> => invoke("help_drawn", { body });
export const helpAnchors = (bodies: string[]): Promise<string[][]> =>
  invoke("help_anchors", { bodies });
