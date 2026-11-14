import { invoke } from "@tauri-apps/api/core";

export const docPlaced = (body: string): Promise<string> =>
  invoke<ArrayBuffer>("doc_placed", { body }).then((bytes) => new TextDecoder().decode(bytes));

export type PlacedNear = {
  readonly spans: readonly (readonly [number, number])[];
  readonly from: number;
  readonly to: number;
  readonly html: string;
};

export const docPlacedNear = (body: string, from: number, to: number): Promise<PlacedNear> =>
  invoke("doc_placed_near", { body, from, to });
