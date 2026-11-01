import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { copied, docCopy, docPlain, type Showing, viewGo } from "../../core";
import { t } from "../../locales";
import type { Choice } from "../Menu";
import { darkNow } from "../theming";
import { PRESETS, type Preset } from "./Stage";

export const ZOOMS = [0.8, 0.9, 1, 1.1, 1.25, 1.4, 1.6];
export const PLAIN = ZOOMS.indexOf(1);

export interface Acts {
  shown: Showing | null;
  body: string;
  dark: boolean | null;
  actual: boolean;
  fail: (problem: unknown) => void;
  look: () => void;
  find: () => void;
  zoom: (step: number) => void;
  side: () => void;
  raw: () => void;
  preview: () => void;
  keep: () => void;
  html: () => void;
  word: () => void;
  quietly: () => void;
  edit: () => void;
  again: () => void;
  code: () => void;
  shade: () => void;
  fit: () => void;
  preset: (one: Preset) => void;
  close: () => void;
}

export const choicesFor = (a: Acts): Choice[] => {
  const shown = a.shown;
  if (shown?.kind === "widget") {
    const from = shown.from;
    return [
      { key: "again", label: t("viewReload"), hint: "Ctrl+R", onPick: a.again },
      { key: "code", label: t("viewCode"), off: !("source" in shown.lent), onPick: a.code },
      {
        key: "doc",
        label: t("viewInDocGo"),
        off: !from,
        onPick: () => from && void viewGo(from.file).catch(a.fail),
      },
      {
        key: "dark",
        label: `${(a.dark ?? darkNow()) ? "✓ " : ""}${t("viewDark")}`,
        apart: true,
        onPick: a.shade,
      },
      {
        key: "actual",
        label: `${a.actual ? "✓ " : ""}${t("viewActual")}`,
        hint: "Ctrl+0",
        onPick: a.fit,
      },
    ];
  }
  const path = shown?.kind === "file" || shown?.kind === "missed" ? shown.path : null;
  // A share has no folder: its path is the link, which opens where it was read.
  const reveal: Choice =
    (shown?.kind === "file" || shown?.kind === "missed") && shown.shared
      ? {
          key: "browse",
          label: t("viewBrowse"),
          onPick: () => void openUrl(shown.path).catch(a.fail),
        }
      : {
          key: "reveal",
          label: t("viewReveal"),
          off: !path,
          onPick: () => path && void revealItemInDir(path).catch(a.fail),
        };
  if (shown?.kind === "missed")
    return [reveal, { key: "retry", label: t("viewRetry"), onPick: a.look }];
  const own: Choice[] =
    shown?.kind === "doc"
      ? [
          {
            key: "copyDoc",
            label: t("viewDuplicate"),
            apart: true,
            onPick: () =>
              void docCopy(shown.paper.id)
                .then((made) => viewGo(made.id))
                .catch(a.fail),
          },
        ]
      : [{ key: "quiet", label: t("viewImportQuiet"), apart: true, onPick: a.quietly }, reveal];
  return [
    { key: "edit", label: t("viewEditIn"), onPick: a.edit },
    { key: "find", label: t("viewFind"), hint: "Ctrl+F", apart: true, onPick: a.find },
    { key: "bigger", label: t("viewBigger"), hint: "Ctrl+=", onPick: () => a.zoom(1) },
    { key: "smaller", label: t("viewSmaller"), hint: "Ctrl+−", onPick: () => a.zoom(-1) },
    ...exportsFor(a),
    ...own,
  ];
};

// The ways out of a document, which the panel's «Export» opens on their own.
export const exportsFor = (a: Acts): Choice[] => [
  {
    key: "copy",
    label: t("viewCopyMd"),
    apart: true,
    onPick: () => void docPlain(a.body).then(copied).catch(a.fail),
  },
  { key: "see", label: t("seePdf"), hint: "Ctrl+P", apart: true, onPick: a.preview },
  { key: "pdf", label: t("toPdf"), onPick: a.keep },
  { key: "html", label: t("toHtml"), onPick: a.html },
  { key: "word", label: t("toWord"), onPick: a.word },
];

export const keysFor = (a: Acts, e: KeyboardEvent): (() => void) | undefined => {
  const key = e.key.toLowerCase();
  const doing: Record<string, () => void> = { w: a.close, r: a.again };
  const kind = a.shown?.kind;
  if (kind === "file" || kind === "doc") {
    Object.assign(doing, {
      f: a.find,
      "\\": a.side,
      p: a.preview,
      "=": () => a.zoom(1),
      "+": () => a.zoom(1),
      "-": () => a.zoom(-1),
      "0": () => a.zoom(0),
    });
    if (e.shiftKey) doing.m = a.raw;
  }
  if (kind === "widget") {
    PRESETS.forEach((one, at) => {
      doing[String(at + 1)] = () => a.preset(one.key);
    });
    doing["0"] = a.fit;
  }
  return doing[key];
};
