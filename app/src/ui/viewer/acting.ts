import { type Showing, viewGo } from "../../core";
import { t } from "../../locales";
import type { Choice } from "../Menu";
import { letterOf } from "../macKeys";
import { darkNow } from "../theming";
import { PRESETS, type Preset } from "./Stage";

export interface Acts {
  shown: Showing | null;
  dark: boolean | null;
  actual: boolean;
  fail: (problem: unknown) => void;
  again: () => void;
  code: () => void;
  shade: () => void;
  fit: () => void;
  preset: (one: Preset) => void;
  close: () => void;
}

export const choicesFor = (a: Acts): Choice[] => {
  const shown = a.shown;
  if (shown?.kind !== "piece") return [];
  const from = shown.from;
  const own: Choice[] = [
    { key: "code", label: t("viewCode"), onPick: a.code },
    {
      key: "doc",
      label: t("viewInDocGo"),
      off: !from,
      onPick: () => from && void viewGo(from.file).catch(a.fail),
    },
  ];
  if (shown.held !== "widget") return own;
  return [
    { key: "again", label: t("viewReload"), hint: "Ctrl+R", onPick: a.again },
    ...own,
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
};

export const keysFor = (a: Acts, e: KeyboardEvent): (() => void) | undefined => {
  const key = letterOf(e) || e.key.toLowerCase();
  const doing: Record<string, () => void> = { w: a.close, r: a.again };
  const shown = a.shown;
  if (shown?.kind === "piece" && shown.held === "widget") {
    PRESETS.forEach((one, at) => {
      doing[String(at + 1)] = () => a.preset(one.key);
    });
    doing["0"] = a.fit;
  }
  return doing[key];
};
