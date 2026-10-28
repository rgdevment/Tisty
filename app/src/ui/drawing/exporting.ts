import { writeImage } from "@tauri-apps/plugin-clipboard-manager";
import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { keepFile } from "../../core";
import { t, type Word } from "../../locales";
import { drawing, grounded } from "../diagrams";
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
// Past these a browser hands back no image at all: WebKit holds a canvas to about 16 million pixels.
const MOST_SIDE = 16384;
const MOST_AREA = 16_000_000;

export interface Drawn {
  readonly svg: string;
  readonly size: Size;
}

const unreadable = () => new Error(t("diagramUnreadable"));
const unpainted = () => new Error(t("diagramUnpainted"));

// A drawing too large to paint at twice its size is painted as large as the browser allows.
export const scaleFor = (size: Size): number =>
  Math.min(
    SCALE,
    MOST_SIDE / size.wide,
    MOST_SIDE / size.tall,
    Math.sqrt(MOST_AREA / (size.wide * size.tall)),
  );

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
  return { svg: grounded(drawn, size, back), size };
};

// Twice its size, so the image stays sharp on a dense screen.
export const rasterized = (drawn: Drawn, type: "image/png" | "image/jpeg"): Promise<Uint8Array> =>
  new Promise((done, fail) => {
    const image = new Image();
    image.onload = () => {
      const scale = scaleFor(drawn.size);
      const canvas = document.createElement("canvas");
      canvas.width = Math.ceil(drawn.size.wide * scale);
      canvas.height = Math.ceil(drawn.size.tall * scale);
      const pen = canvas.getContext("2d");
      if (!pen) return fail(unpainted());
      pen.scale(scale, scale);
      pen.drawImage(image, 0, 0, drawn.size.wide, drawn.size.tall);
      canvas.toBlob(
        (blob) =>
          blob
            ? blob.arrayBuffer().then((bytes) => done(new Uint8Array(bytes)), fail)
            : fail(unpainted()),
        type,
        0.92,
      );
    };
    image.onerror = () => fail(unpainted());
    image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(drawn.svg)}`;
  });

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
  // Asked only when a file is wanted: finding which diagram it is may read the document.
  readonly name: () => Promise<string>;
  readonly dark: boolean;
  readonly said: (text: string) => void;
  readonly failed: (problem: unknown) => void;
}

const fontNow = () => getComputedStyle(document.body).fontFamily || "sans-serif";

// One list for the window and the page, so both offer exactly the same.
export const exportChoices = (to: Exporting): Choice[] => [
  ...(Object.keys(WAYS) as Way[]).map((way) => ({
    key: way,
    label: t(WORDS[way]),
    onPick: () =>
      void to
        .name()
        .then((name) => exported(to.source, way, name, fontNow()))
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
