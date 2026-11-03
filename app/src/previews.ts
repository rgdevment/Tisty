import { ending, hosting } from "tisty-editor/read";
import { DOC } from "./markdown";

export type Preview =
  | { as: "video"; at: string }
  | { as: "audio"; at: string }
  | { as: "file"; at: string; kind: string }
  | { as: "doc"; id: string }
  | { as: "web"; at: string; host: string };

export const SEEABLE = ["png", "jpg", "jpeg", "gif", "svg", "webp", "avif"];

// WebKit draws HEIC itself and a Windows with the HEIF decoder draws it for Tisty; elsewhere it stays a file.
let heicDrawn = navigator.userAgent.includes("Macintosh");

export const drawsHeic = (yes: boolean): void => {
  heicDrawn = yes;
};

export const heicReadable = (): boolean => heicDrawn;

const WATCHABLE = ["mp4", "webm", "m4v", "mov", "ogv"];
const HEARABLE = ["mp3", "m4a", "wav", "ogg", "oga", "aac", "flac"];

const outside = (href: string): boolean =>
  /^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith("/") || href.startsWith("\\\\");

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
// The core's own test for an attachment's name: a stamp of hex after the last dash, all in lower case.
export const kept = (href: string): boolean => {
  const said = href.trim().replace(/^<|>$/g, "").split(/[?#]/)[0];
  const leaf = said.startsWith("attachments/") ? (said.split("/").pop() ?? "") : said;
  if (!leaf || leaf.includes("/") || leaf.includes("\\") || leaf !== leaf.toLowerCase())
    return false;
  const stamp = leaf.split(".")[0].split("-").pop() ?? "";
  return /^[0-9a-f]{8,16}$/.test(stamp);
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
  if (!kind && !kept(at)) return null;
  return { as: "file", at, kind };
};

export const pictured = (href: string): boolean => {
  const kind = ending(href);
  return !href.startsWith(DOC) && (SEEABLE.includes(kind) || (kind === "heic" && heicDrawn));
};

export { weighed } from "tisty-editor/read";

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
