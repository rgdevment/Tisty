import { drawnGlyph, piece, SHEET_GLYPHS, type SheetGlyph } from "tisty-sheet";

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

// The core's own test for an attachment's name: a stamp of hex after the last dash, all in lower case.
export const kept = (href: string): boolean => {
  const said = href.trim().replace(/^<|>$/g, "").split(/[?#]/)[0];
  const leaf = said.startsWith("attachments/") ? (said.split("/").pop() ?? "") : said;
  if (!leaf || leaf.includes("/") || leaf.includes("\\") || leaf !== leaf.toLowerCase())
    return false;
  const stamp = leaf.split(".")[0].split("-").pop() ?? "";
  return /^[0-9a-f]{8,16}$/.test(stamp);
};

/// Whether a picture stands on a line of its own, where a file is a card rather than a chip.
export const alone = (img: Element): boolean => {
  const held = img.parentElement;
  return held?.tagName === "P" && held.children.length === 1 && !(held.textContent ?? "").trim();
};

/// A file as Tisty draws it in place of its picture: a card on a line of its own, a chip within words; the host links it.
export const fileCard = (
  page: Document,
  img: Element,
  kind: string,
  name: string,
  said: string,
): {
  readonly card: HTMLAnchorElement;
  readonly meta?: HTMLElement;
  readonly kindIs?: HTMLElement;
} => {
  const card = page.createElement("a");
  card.dataset.family = family(kind);
  if (!alone(img)) {
    card.className = "card";
    card.textContent = name;
    img.replaceWith(card);
    return { card };
  }
  card.className = "doc-card is-file";
  const tile = piece(page, "card-tile");
  tile.append(drawnGlyph(page, fileGlyph(kind)));
  const meta = piece(page, "card-meta");
  const kindIs = piece(page, "kind", said);
  meta.append(kindIs);
  card.append(tile, piece(page, "card-name", name), piece(page, "card-blurb"), meta);
  img.replaceWith(card);
  return { card, meta, kindIs };
};
