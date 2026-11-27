import { invoke } from "@tauri-apps/api/core";

export interface Admitted {
  step: "idle" | "waiting" | "compare" | "trusted" | "done";
  code: string | null;
  by: string | null;
}

export interface Newcomer {
  device: string;
  name: string;
  at: number;
  taken: boolean;
}

export const admittingAsk = (): Promise<void> => invoke("admitting_ask");
export const admittingShown = (): Promise<Admitted> => invoke("admitting_shown");
export const admittingTrust = (code: string): Promise<void> => invoke("admitting_trust", { code });
export const admittingCancel = (): Promise<void> => invoke("admitting_cancel");
export const admittingLook = (): Promise<Newcomer[]> => invoke("admitting_look");
export const admittingOffer = (device: string): Promise<void> =>
  invoke("admitting_offer", { device });
export const admittingGrant = (device: string, code: string): Promise<void> =>
  invoke("admitting_grant", { device, code });
export const admittingRefuse = (device: string): Promise<void> =>
  invoke("admitting_refuse", { device });
