import { invoke } from "@tauri-apps/api/core";
import type { Viewed } from "./core";

export type Held = "widget" | "formula" | "inlineFormula" | "table" | "front";

export type PiecePlace = { readonly at: number; readonly nth: number };

export type PieceShowing = {
  kind: "piece";
  held: Held;
  source: string;
  from: Viewed | null;
  code: boolean;
  writable: boolean;
};

// Only a widget brings its words: the core reads every other piece from the file at its place.
export const viewPiece = (
  held: Held,
  place?: PiecePlace,
  from?: Viewed,
  source?: string,
  code = true,
): Promise<void> =>
  invoke("view_piece", {
    held,
    source: source ?? null,
    from: from ?? null,
    at: place?.at ?? null,
    nth: place?.nth ?? null,
    code,
  });

export const pieceSave = (now: string): Promise<void> => invoke("piece_save", { now });
