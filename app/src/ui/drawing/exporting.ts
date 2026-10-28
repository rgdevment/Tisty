import { writeImage } from "@tauri-apps/plugin-clipboard-manager";
import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { keepFile } from "../../core";
import { t, type Word } from "../../locales";
import { drawing } from "../diagrams";
import type { Choice } from "../Menu";
import { type Size, sizeOf } from "./framing";
import { PALETTES, svgOf } from "./picture";
import { pictureOf } from "./pictures";

export type Way = "pngClear" | "pngLight" | "pngDark" | "jpgLight" | "jpgDark" | "svg";

const WAYS: Record<Way, { kind: "png" | "jpg" | "svg"; dark: boolean; ground: boolean }> = {
  pngClear: { kind: "png", dark: false, ground: false },
  pngLight: { kind: "png", dark: false, ground: true },
  pngDark: { kind: "png", dark: true, ground: true },
  jpgLight: { kind: "jpg", dark: false, ground: true },
  jpgDark: { kind: "jpg", dark: true, ground: true },
  svg: { kind: "svg", dark: false, ground: false },
};

const WORDS: Record<Way, Word> = {
  pngClear: "exportPngClear",
  pngLight: "exportPngLight",
  pngDark: "exportPngDark",
  jpgLight: "exportJpgLight",
  jpgDark: "exportJpgDark",
  svg: "exportSvg",
};

const SCALE = 2;

export interface Drawn {
  readonly svg: string;
  readonly size: Size;
}

const unreadable = () => new Error(t("diagramUnreadable"));

/** The drawing as a file: Tisty's own for the kinds its window draws, Mermaid's for the rest. */
export const svgFor = async (
  source: string,
  dark: boolean,
  ground: boolean,
  font: string,
): Promise<Drawn> => {
  const palette = PALETTES[dark ? "dark" : "light"];
  const back = ground ? palette.sheet : null;
  const own = pictureOf(source);
  if (own) return { svg: svgOf(own, palette, back, font), size: own.size };
  const drawn = await drawing(source, dark);
  const size = drawn && sizeOf(drawn);
  if (!drawn || !size) throw unreadable();
  // Mermaid draws to the width of its place; a file needs a size of its own.
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${size.wide}" height="${size.tall}" viewBox="0 0 ${size.wide} ${size.tall}">${back ? `<rect width="100%" height="100%" fill="${back}"/>` : ""}${drawn}</svg>`;
  return { svg, size };
};

/** The drawing as an image at twice its size, so it stays sharp on any screen. */
export const rasterized = (drawn: Drawn, type: "image/png" | "image/jpeg"): Promise<Uint8Array> =>
  new Promise((done, fail) => {
    const image = new Image();
    image.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = Math.ceil(drawn.size.wide * SCALE);
      canvas.height = Math.ceil(drawn.size.tall * SCALE);
      const pen = canvas.getContext("2d");
      if (!pen) return fail(unreadable());
      pen.scale(SCALE, SCALE);
      pen.drawImage(image, 0, 0, drawn.size.wide, drawn.size.tall);
      canvas.toBlob(
        (blob) =>
          blob
            ? blob.arrayBuffer().then((bytes) => done(new Uint8Array(bytes)), fail)
            : fail(unreadable()),
        type,
        0.92,
      );
    };
    image.onerror = () => fail(unreadable());
    image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(drawn.svg)}`;
  });

/** The name the save dialog offers: the document's title, numbered from its second diagram on. */
export const fileName = (title: string, nth: number): string => {
  const base = title.replace(/[\\/:*?"<>|\p{Cc}]/gu, "").trim() || t("untitledDoc");
  return nth > 0 ? `${base}-${nth + 1}` : base;
};

export const exported = async (source: string, way: Way, name: string, font: string) => {
  const how = WAYS[way];
  const drawn = await svgFor(source, how.dark, how.ground, font);
  const path = await intoFile({
    defaultPath: `${name}.${how.kind}`,
    filters: [{ name: how.kind.toUpperCase(), extensions: [how.kind] }],
  });
  if (!path) return false;
  const bytes =
    how.kind === "svg"
      ? new TextEncoder().encode(drawn.svg)
      : await rasterized(drawn, how.kind === "png" ? "image/png" : "image/jpeg");
  await keepFile(path, Array.from(bytes));
  return true;
};

// A copy is pasted wherever, so it keeps the ground of the theme it was copied in.
export const copiedImage = async (source: string, dark: boolean, font: string) =>
  writeImage(await rasterized(await svgFor(source, dark, true, font), "image/png"));

export interface Exporting {
  readonly source: string;
  readonly name: string;
  readonly dark: boolean;
  readonly said: (text: string) => void;
  readonly failed: (problem: unknown) => void;
}

const fontNow = () => getComputedStyle(document.body).fontFamily || "sans-serif";

/** The same choices in the window and on the page: each file, then a copy for pasting. */
export const exportChoices = (to: Exporting): Choice[] => [
  ...(Object.keys(WAYS) as Way[]).map((way) => ({
    key: way,
    label: t(WORDS[way]),
    onPick: () =>
      void exported(to.source, way, to.name, fontNow())
        .then((kept) => kept && to.said(t("diagramExported")))
        .catch(to.failed),
  })),
  {
    key: "copy",
    label: t("copyAsImage"),
    apart: true,
    onPick: () =>
      void copiedImage(to.source, to.dark, fontNow())
        .then(() => to.said(t("diagramCopiedImage")))
        .catch(to.failed),
  },
];
