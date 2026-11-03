import { SHEET_GLYPHS, type SheetGlyph } from "./sheetGlyphs";

// What a file is by its name: its kind, the family it is drawn as, and the glyph its card wears.
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

const GLYPHS: Record<string, SheetGlyph> = {
  pdf: "page",
  word: "page",
  text: "page",
  sheet: "sheet",
  data: "table",
  slides: "book",
  archive: "file-archive",
  page: "code",
  video: "file-video",
  audio: "file-audio",
};

export const ending = (href: string): string => {
  const name = href.split(/[?#]/)[0].split("/").pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
};

// The same few glyphs wherever a file is drawn, so a page that names no icon never loads them all.
export const fileGlyph = (kind: string): string => SHEET_GLYPHS[GLYPHS[family(kind)] ?? "page"];
