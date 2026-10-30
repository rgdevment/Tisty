import { writeImage } from "@tauri-apps/plugin-clipboard-manager";
import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { rasterized, svgFor } from "tisty-diagrams/read";
import { drawing } from "tisty-editor/read";
import { keepFile } from "../../core";
import { t, type Word } from "../../locales";
import type { Choice } from "../Menu";

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

export const fileName = (title: string, nth: number): string => {
  const base = title.replace(/[\\/:*?"<>|\p{Cc}]/gu, "").trim() || t("untitledDoc");
  return nth > 0 ? `${base}-${nth + 1}` : base;
};

export const exported = async (source: string, way: Way, name: string, font: string) => {
  const how = WAYS[way];
  const drawn = await svgFor(source, how.dark, how.ground, font, drawing);
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
  writeImage(await rasterized(await svgFor(source, dark, true, font, drawing), "image/png"));

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
