import { matched } from "tisty-editor/read";
import { type Hands, hereChoices } from "./docMenus";
import { waysOut } from "./leaving";
import { t } from "./locales";
import { FIND_KEY } from "./ui/finding";
import type { Choice } from "./ui/Menu";
import { shortcut } from "./ui/macKeys";
import type { Pickable } from "./ui/Picker";
import { FOLD_KEY } from "./ui/SideEdge";
import { WHOLE_ASKED } from "./ui/wholeKeys";
import type { Chosen } from "./views";

export const ACTIONS = ">";

export interface Acting {
  chosen: Chosen;
  here: string | null | undefined;
  hands: Hands;
  board: string | null;
  folded: boolean;
  choose: (next: Chosen) => void;
  library: () => void;
  diary: () => void;
  capture: () => void;
  find: () => void;
  fold: () => void;
}

const WHOLE_KEY = shortcut("E");

const choices = (group: string, all: Choice[]): Pickable[] =>
  all
    .filter((one) => !one.off && one.onPick)
    .map((one) => ({
      key: one.key,
      glyph: one.icon,
      label: one.label,
      keys: one.keys,
      group,
      onPick: one.onPick as () => void,
    }));

export function actions(a: Acting): Pickable[] {
  const go = (key: string, label: string, onPick: () => void, keys?: string): Choice => ({
    key,
    icon: "→",
    label,
    keys,
    onPick,
  });
  const reading = a.chosen.named === "docs" && a.chosen.doc && a.hands.showing === a.chosen.doc;
  const board = a.board;
  return [
    ...choices(t("paletteGo"), [
      go("today", t("todayIs"), () => a.choose({ named: "tasks", slice: "today" })),
      go("upcoming", t("upcoming"), () => a.choose({ named: "tasks", slice: "upcoming" })),
      go("repeating", t("repeating"), () => a.choose({ named: "tasks", slice: "repeating" })),
      go("all", t("allTasks"), () => a.choose({ named: "tasks", slice: "all" })),
      go("quadrants", t("quadrants"), () => a.choose({ named: "quadrants" })),
      go("spread", t("spread"), () => a.choose({ named: "spread" })),
      go("lists", t("lists"), () => a.choose({ named: "lists" })),
      go("tags", t("tags"), () => a.choose({ named: "tags" })),
      go("archive", t("archive"), () => a.choose({ named: "archive" })),
      go("weekly", t("weekly"), () => a.choose({ named: "weekly" })),
      go("library", t("library"), a.library),
      ...(board
        ? [go("brainstorm", t("brainstorm"), () => a.choose({ named: "docs", doc: board }))]
        : []),
      go("diary", t("paletteDiary"), a.diary),
      go("keeping", t("keeping"), () => a.choose({ named: "keeping" })),
      go("help", t("help"), () => a.choose({ named: "help" }), "F1"),
    ]),
    ...choices(t("paletteMake"), [
      { key: "capture", icon: "+", label: t("captureNew"), onPick: a.capture },
      ...hereChoices(a.hands, a.here ?? undefined),
    ]),
    ...choices(t("paletteWindow"), [
      { key: "find", icon: "⌕", label: t("sideSearch"), keys: FIND_KEY, onPick: a.find },
      {
        key: "fold",
        icon: "◧",
        label: t(a.folded ? "sideUnfold" : "sideFold"),
        keys: FOLD_KEY,
        onPick: a.fold,
      },
    ]),
    ...(reading
      ? choices(t("paletteDoc"), [
          {
            key: "whole",
            icon: "⇄",
            label: t("paletteWhole"),
            keys: WHOLE_KEY,
            onPick: () => window.dispatchEvent(new CustomEvent(WHOLE_ASKED)),
          },
          ...waysOut(false),
        ])
      : []),
  ];
}

export const sifted = (all: Pickable[], asked: string): Pickable[] =>
  asked.trim() ? all.filter((one) => matched(`${one.group ?? ""} ${one.label}`, asked)) : all;
