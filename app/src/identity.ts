import { invoke } from "@tauri-apps/api/core";

export interface Identity {
  held: boolean;
  elsewhere: boolean;
  fingerprint: string | null;
  suffix: string | null;
  confirmed: boolean;
  alias: string | null;
  machines: { id: string; name: string; here: boolean; ours: boolean }[];
}

export const identityShown = (): Promise<Identity> => invoke("identity_shown");
export const identityPhrase = (): Promise<string> => invoke("identity_phrase");
export const identityConfirm = (said: [number, string][]): Promise<number[]> =>
  invoke("identity_confirm", { said });
export const identityBegin = (): Promise<string> => invoke("identity_begin");
export const identityRecover = (said: string): Promise<void> =>
  invoke("identity_recover", { said });
