import type { Word } from "./locales";

export interface Cloud {
  id: string;
  shown: string;
  maker: string;
  name: Word;
  why: Word;
  letter: string;
  hue: string;
  storage?: string;
  variant?: string;
  // Signed in to with an address and a password typed here, so no browser opens and nothing can be revoked everywhere.
  typed?: boolean;
}

export const CLOUDS: Cloud[] = [
  {
    id: "google",
    shown: "Google Drive",
    maker: "Google",
    name: "cloudGoogle",
    why: "cloudGoogleWhy",
    letter: "G",
    hue: "bg-[#1a73e8]",
    storage: "https://drive.google.com/settings/storage",
  },
  {
    id: "onedrive",
    shown: "OneDrive",
    maker: "Microsoft",
    name: "cloudOneDrive",
    why: "cloudOneDriveWhy",
    letter: "O",
    hue: "bg-[#0364b8]",
    storage: "https://onedrive.live.com/?v=managestorage",
    variant: "OneDrive",
  },
  {
    id: "webdav",
    shown: "WebDAV",
    maker: "WebDAV",
    name: "cloudWebDav",
    why: "cloudWebDavWhy",
    letter: "W",
    hue: "bg-[#3d5a80]",
    variant: "WebDav",
    typed: true,
  },
  {
    id: "dropbox",
    shown: "Dropbox",
    maker: "Dropbox",
    name: "cloudDropbox",
    why: "cloudComing",
    letter: "D",
    hue: "bg-soft",
  },
];

export const cloudOf = (id?: string | null): Cloud | undefined =>
  CLOUDS.find((one) => one.id === id);

export const typed = (id?: string | null): boolean => cloudOf(id)?.typed === true;
