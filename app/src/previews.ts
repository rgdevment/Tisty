import { DOC } from "./markdown";

export type Preview =
  | { as: "video"; at: string }
  | { as: "audio"; at: string }
  | { as: "file"; at: string; kind: string }
  | { as: "doc"; id: string }
  | { as: "web"; at: string; host: string };

export const KINDS: Record<string, string> = {
  pdf: "kindPdf",
  md: "kindMarkdown",
  markdown: "kindMarkdown",
  txt: "kindText",
  rtf: "kindText",
  doc: "kindWord",
  docx: "kindWord",
  odt: "kindWord",
  xls: "kindExcel",
  xlsx: "kindExcel",
  ods: "kindExcel",
  ppt: "kindSlides",
  pptx: "kindSlides",
  odp: "kindSlides",
  zip: "kindArchive",
  rar: "kindArchive",
  "7z": "kindArchive",
  gz: "kindArchive",
  tar: "kindArchive",
  csv: "kindSheet",
  json: "kindData",
  xml: "kindData",
  yml: "kindData",
  yaml: "kindData",
  toml: "kindData",
  png: "kindImage",
  jpg: "kindImage",
  jpeg: "kindImage",
  gif: "kindImage",
  svg: "kindImage",
  html: "kindPage",
  htm: "kindPage",
  webp: "kindImage",
  avif: "kindImage",
  heic: "kindImage",
  mp4: "kindVideo",
  webm: "kindVideo",
  m4v: "kindVideo",
  mov: "kindVideo",
  ogv: "kindVideo",
  mp3: "kindAudio",
  m4a: "kindAudio",
  wav: "kindAudio",
  ogg: "kindAudio",
  oga: "kindAudio",
  aac: "kindAudio",
  flac: "kindAudio",
};

const FAMILIES: Record<string, string> = {
  kindPdf: "pdf",
  kindWord: "word",
  kindExcel: "sheet",
  kindSheet: "sheet",
  kindSlides: "slides",
  kindArchive: "archive",
  kindData: "data",
  kindImage: "image",
  kindPage: "page",
  kindVideo: "video",
  kindAudio: "audio",
  kindMarkdown: "text",
  kindText: "text",
};

export const family = (kind: string): string => FAMILIES[KINDS[kind] ?? ""] ?? "plain";

// Only WebKit draws HEIC in an <img>; elsewhere it would land as a broken picture.
const DRAWS_HEIC = navigator.userAgent.includes("Macintosh");

export const SEEABLE = ["png", "jpg", "jpeg", "gif", "svg", "webp", "avif"].concat(
  DRAWS_HEIC ? ["heic"] : [],
);

const WATCHABLE = ["mp4", "webm", "m4v", "mov", "ogv"];
const HEARABLE = ["mp3", "m4a", "wav", "ogg", "oga", "aac", "flac"];

const outside = (href: string): boolean =>
  /^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith("/") || href.startsWith("\\\\");

export const ending = (href: string): string => {
  const name = href.split(/[?#]/)[0].split("/").pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
};

export const named = (href: string): string => {
  const raw = href.split(/[?#]/)[0].split("/").pop() ?? href;
  try {
    return decodeURI(raw);
  } catch {
    return raw;
  }
};

/// Only the two schemes a person can be shown safely, and never the machine's own paths: what
/// comes back is a name and a host, never something a card would go and load.
export const hosting = (href: string): string | null => {
  try {
    const said = new URL(href);
    if (said.protocol !== "http:" && said.protocol !== "https:") return null;
    return said.hostname.replace(/^www\./, "") || null;
  } catch {
    return null;
  }
};

export const previewOf = (href: string): Preview | null => {
  const at = href.trim();
  if (!at) return null;

  const paper = at.startsWith(DOC) ? at.slice(DOC.length) : null;
  if (paper) return { as: "doc", id: paper };

  if (outside(at)) {
    const host = hosting(at);
    return host ? { as: "web", at, host } : null;
  }

  const kind = ending(at);
  if (WATCHABLE.includes(kind)) return { as: "video", at };
  if (HEARABLE.includes(kind)) return { as: "audio", at };
  if (!kind && !at.startsWith("attachments/")) return null;
  return { as: "file", at, kind };
};

export const pictured = (href: string): boolean =>
  !href.startsWith(DOC) && SEEABLE.includes(ending(href));

export const weighed = (bytes: number): string => {
  const units = ["B", "kB", "MB", "GB"];
  let step = 0;
  let left = bytes;
  while (left >= 1000 && step < units.length - 1) {
    left /= 1000;
    step += 1;
  }
  if (step === 0) return `${Math.round(left)} ${units[step]}`;
  // Rounding to one decimal can reach the next unit, and «1000.0 kB» reads a step behind.
  if (Number(left.toFixed(1)) >= 1000 && step < units.length - 1) {
    left /= 1000;
    step += 1;
  }
  return `${left.toFixed(1)} ${units[step]}`;
};

export const MANY = 150;

export const crowd = (body: string): number => {
  let count = 0;
  let at = body.indexOf("](");
  while (at >= 0) {
    const end = body.indexOf(")", at + 2);
    if (end < 0) break;
    const href = body
      .slice(at + 2, end)
      .trim()
      .replace(/^<|>$/g, "");
    const seen = previewOf(href);
    if (seen && seen.as !== "web") count += 1;
    at = body.indexOf("](", end + 1);
  }
  return count;
};
