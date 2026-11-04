import { invoke } from "@tauri-apps/api/core";
import { fill, t } from "./locales";

export type Told =
  | {
      what: "pictureAsFile" | "pictureAsWords" | "pictureAsLink" | "fileAsWords";
      document: string;
      name: string;
    }
  | { what: "notDrawn"; document: string; many: number }
  | { what: "pageLeftBehind" | "languageLeftBehind"; document: string }
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
    case "notDrawn":
      return fill("toldNotDrawn", String(told.many), told.document);
    case "pageLeftBehind":
      return fill("toldPageLeftBehind", told.document);
    case "languageLeftBehind":
      return fill("toldLanguageLeftBehind", told.document);
    case "nameLeftOut":
      return t("toldNameLeftOut");
  }
};
