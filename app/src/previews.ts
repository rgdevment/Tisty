import { DOC } from "./markdown";

export type Preview =
  | { as: "video"; at: string }
  | { as: "audio"; at: string }
  | { as: "file"; at: string; kind: string }
  | { as: "doc"; id: string };

export const KINDS: Record<string, string> = {
  pdf: "PDF",
  md: "Markdown",
  txt: "Texto",
  doc: "Word",
  docx: "Word",
  xls: "Excel",
  xlsx: "Excel",
  ppt: "PowerPoint",
  pptx: "PowerPoint",
  zip: "Archivo comprimido",
  csv: "Hoja de datos",
  json: "JSON",
  png: "Imagen",
  jpg: "Imagen",
  jpeg: "Imagen",
  gif: "Imagen",
  svg: "Imagen",
  heic: "Imagen",
};

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

export const previewOf = (href: string): Preview | null => {
  const at = href.trim();
  if (!at) return null;

  const paper = at.startsWith(DOC) ? at.slice(DOC.length) : null;
  if (paper) return { as: "doc", id: paper };

  if (outside(at)) return null;

  const kind = ending(at);
  if (WATCHABLE.includes(kind)) return { as: "video", at };
  if (HEARABLE.includes(kind)) return { as: "audio", at };
  if (!kind) return null;
  return { as: "file", at, kind };
};

export const weighed = (bytes: number): string => {
  const units = ["B", "kB", "MB", "GB"];
  let step = 0;
  let left = bytes;
  while (left >= 1000 && step < units.length - 1) {
    left /= 1000;
    step += 1;
  }
  return step === 0 ? `${Math.round(left)} ${units[step]}` : `${left.toFixed(1)} ${units[step]}`;
};
