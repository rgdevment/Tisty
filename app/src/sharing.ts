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
  | { what: "languageOverTheTop"; language: string; top: number }
  | { what: "nameLeftOut" | "takenBackNotKept" };

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
  | { state: "stillAccepted" }
  | { state: "toAccept"; version: string; text: string; acceptedBefore: string | null };

export interface FolioLimits {
  ttl_days?: number[];
  shares_at_most?: number;
  new_shares_a_day?: number;
  pictures_at_most?: number;
  picture_bytes_at_most?: number;
  pictures_bytes_at_most?: number;
  files_at_most?: number;
  files_bytes_at_most?: number;
  envelope_bytes_at_most?: number;
}

export interface ShareLook {
  signs: boolean;
  secrets: { named: string; under: string | null }[];
  citedSecrets: { named: string; under: string | null }[];
  name: string | null;
  alias: string | null;
  carrying: {
    pages: number;
    pictures: number;
    picturesBytes: number;
    largestPicture: number;
    largestPictureName: string | null;
    mainBytes: number;
    files: number;
    filesBytes: number;
  };
  cited: string[];
  shared: { doc: string; title: string }[];
  limits: FolioLimits;
  told: Told[];
  seen: string | null;
  newSharesToday: number;
  newSharesFull: boolean;
  heldThere: string[];
}

export interface Named {
  name: string;
  checked: boolean;
}

export interface ShareCard {
  read: string;
  edit: string | null;
  version: number;
  taken: boolean;
  held: boolean;
  told: Told[];
  owner: Named | null;
  signer: Named | null;
  editors: Named[];
  signsAs: { now: string | null; other: string | null } | null;
}

export type Synced =
  | { state: "unchanged"; told: Told[]; termsUnchecked: boolean }
  | { state: "secrets"; place: string; print: string }
  | { state: "gone" }
  | { state: "sent"; version: number | null; told: Told[]; termsUnchecked: boolean }
  | { state: "otherVersion"; version: number; signer: string; doubtful: boolean };

export interface ShareWaiting {
  until: number | null;
  pending: boolean;
}

export interface ShareStatus {
  diesOn: number;
  paused: boolean;
  blocked: boolean;
  reads: number;
  appealed: boolean;
  mayCorrect: boolean;
  fixBy: number | null;
}

export type SharingNow =
  | { state: "open" }
  | { state: "closed"; note: Record<string, string | undefined> | null }
  | { state: "unreached" };

export interface NotePart {
  at: number;
  text: string;
  url?: string;
}

const NOTE_LINK = /(?<!!)\[([^\]\n]+)\]\((https:\/\/[^\s()]+)\)/g;

const noteLinkAllowed = (said: string) => {
  try {
    const url = new URL(said);
    const ours =
      url.hostname === "tisty.cl" ||
      url.hostname === "tisty.app" ||
      url.hostname.endsWith(".tisty.app");
    return ours && !url.username && !url.password && !url.port;
  } catch {
    return false;
  }
};

export const noteParts = (note: string): NotePart[] => {
  const parts: NotePart[] = [];
  let done = 0;
  for (const found of note.matchAll(NOTE_LINK)) {
    const [whole, text, url] = found;
    if (!noteLinkAllowed(url)) continue;
    if (found.index > done) parts.push({ at: done, text: note.slice(done, found.index) });
    parts.push({ at: found.index, text, url });
    done = found.index + whole.length;
  }
  if (done < note.length) parts.push({ at: done, text: note.slice(done) });
  return parts;
};

export const sharingNow = (): Promise<SharingNow> => invoke("folio_sharing_now");

export const termsNow = (): Promise<TermsNow> => invoke("folio_terms_standing");

export const termsDrawn = (text: string): Promise<string> => invoke("folio_terms_drawn", { text });

export const termsAccepted = (version: string): Promise<void> =>
  invoke("folio_terms_accept", { version });

export const shareLook = (id: string): Promise<ShareLook> => invoke("doc_share_look", { id });

export const shareCard = (id: string): Promise<ShareCard | null> =>
  invoke("doc_share_card", { id });

export const docShared = (
  id: string,
  chosen: { cited: string[]; signedAs?: string; editable: boolean; ttlDays: number; seen?: string },
): Promise<ShareCard> => invoke("doc_shared", { id, chosen });

export const shareSync = (id: string, accepted?: string): Promise<Synced> =>
  invoke("doc_share_sync", { id, accepted });

export const shareSignAs = (id: string, name: string): Promise<ShareCard> =>
  invoke("doc_share_sign_as", { id, name });

export const shareWaiting = (id: string, look: boolean): Promise<ShareWaiting> =>
  invoke("doc_share_waiting", { id, look });

export const shareStatus = (id: string): Promise<ShareStatus> => invoke("doc_share_status", { id });

export const sharePause = (id: string, paused: boolean): Promise<void> =>
  invoke("doc_share_pause", { id, paused });

export const shareAppeal = (id: string): Promise<boolean> => invoke("doc_share_appeal", { id });

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
    case "languageOverTheTop":
      return fill("toldLanguageOverTheTop", told.language, String(told.top));
    case "citedLeftBehind":
      return fill("toldCitedLeftBehind", told.document);
    case "nameLeftOut":
      return t("toldNameLeftOut");
    case "takenBackNotKept":
      return t("toldTakenBackNotKept");
  }
};
