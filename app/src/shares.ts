import { invoke } from "@tauri-apps/api/core";

/** What the person chose in «Share»: a password, the edit link, a signature and a shorter life. */
export interface ShareChosen {
  password: string | null;
  editable: boolean;
  signed: boolean;
  idleDays: number | null;
}

/** How a share stands on the server, when it could be asked. */
export interface ShareInfo {
  latest: number | null;
  reads: number;
  ends: number;
  paused: boolean;
  blocked: boolean;
  gated: boolean;
}

export interface ShareCard {
  read: string;
  edit: string | null;
  gated: boolean;
  signed: boolean;
  version: number;
  info: ShareInfo | null;
}

export const shareDoc = (file: string, chosen: ShareChosen): Promise<ShareCard> =>
  invoke("share_doc", { file, chosen });
export const shareCard = (file: string): Promise<ShareCard | null> =>
  invoke("share_card", { file });
export const shareStop = (file: string): Promise<void> => invoke("share_stop", { file });
