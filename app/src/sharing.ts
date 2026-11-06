import { invoke } from "@tauri-apps/api/core";
import { fill, t } from "./locales";

export type Told =
  | {
      what: "pictureAsFile" | "pictureAsWords" | "pictureAsLink" | "fileAsWords";
      document: string;
      name: string;
    }
  | { what: "notDrawn"; document: string; many: number }
  | { what: "pictureNotKept"; document: string; name: string }
  | { what: "pageLeftBehind" | "citedLeftBehind"; document: string }
  | { what: "languageLeftBehind"; language: string }
  | { what: "nameLeftOut" };

export interface Sealed {
  envelopes: number;
  bytes: number;
  told: Told[];
}

export interface Sharing {
  public: boolean;
  cited: string[];
  signedAs?: string;
}

export const docSealed = (
  id: string,
  { public: isPublic, cited, signedAs }: Sharing,
): Promise<Sealed> => invoke("doc_sealed", { id, public: isPublic, cited, signedAs });

export type TermsNow =
  | { state: "accepted" }
  | { state: "toAccept"; version: string; text: string; acceptedBefore: string | null };

export interface FolioLimits {
  ttl_days?: number[];
  shares_at_most?: number;
  pictures_at_most?: number;
  pictures_bytes_at_most?: number;
  files_at_most?: number;
  files_bytes_at_most?: number;
}

export interface ShareLook {
  secrets: { named: string; under: string | null }[];
  name: string | null;
  alias: string | null;
  carrying: {
    pages: number;
    pictures: number;
    picturesBytes: number;
    files: number;
    filesBytes: number;
  };
  cited: string[];
  shared: { doc: string; title: string }[];
  limits: FolioLimits;
  told: Told[];
}

export interface ShareCard {
  read: string;
  edit: string | null;
  version: number;
}

export type Synced =
  | { state: "unchanged" }
  | { state: "sent"; version: number }
  | { state: "otherVersion"; version: number };

export interface ShareStatus {
  diesOn: number;
  paused: boolean;
  blocked: boolean;
  reads: number;
}

export const termsNow = (): Promise<TermsNow> => invoke("folio_terms_standing");

export const termsAccepted = (version: string): Promise<void> =>
  invoke("folio_terms_accept", { version });

export const shareLook = (id: string): Promise<ShareLook> => invoke("doc_share_look", { id });

export const shareCard = (id: string): Promise<ShareCard | null> =>
  invoke("doc_share_card", { id });

export const docShared = (
  id: string,
  chosen: { cited: string[]; signedAs?: string; editable: boolean; ttlDays: number },
): Promise<ShareCard> => invoke("doc_shared", { id, ...chosen });

export const shareSync = (id: string): Promise<Synced> => invoke("doc_share_sync", { id });

export const shareStatus = (id: string): Promise<ShareStatus> => invoke("doc_share_status", { id });

export const sharePause = (id: string, paused: boolean): Promise<void> =>
  invoke("doc_share_pause", { id, paused });

export const shareStop = (id: string): Promise<void> => invoke("doc_share_stop", { id });

export const toldPlainly = (told: Told): string => {
  switch (told.what) {
    case "pictureAsFile":
      return fill("toldPictureAsFile", told.name, told.document);
    case "pictureAsWords":
      return fill("toldPictureAsWords", told.name, told.document);
    case "pictureAsLink":
      return fill("toldPictureAsLink", told.name, told.document);
    case "fileAsWords":
      return fill("toldFileAsWords", told.name, told.document);
    case "pictureNotKept":
      return told.name
        ? fill("toldPictureNotKept", told.name, told.document)
        : fill("toldSomePictureNotKept", told.document);
    case "notDrawn":
      return told.many === 1
        ? fill("toldOneNotDrawn", told.document)
        : fill("toldNotDrawn", String(told.many), told.document);
    case "pageLeftBehind":
      return fill("toldPageLeftBehind", told.document);
    case "languageLeftBehind":
      return fill("toldLanguageLeftBehind", told.language);
    case "citedLeftBehind":
      return fill("toldCitedLeftBehind", told.document);
    case "nameLeftOut":
      return t("toldNameLeftOut");
  }
};
