import { type List, listArchive, listDrop, listLook, listRename } from "./core";
import { fill, t } from "./locales";
import { sure } from "./sure";
import type { Choice } from "./ui/Menu";

export interface ListHands {
  edit: (list: List) => void;
  gone: (list: string) => void;
  fail: (problem: unknown) => void;
}

export const settleList = (
  list: List,
  called: string,
  drawn?: string,
  colour?: string,
): Promise<unknown> =>
  Promise.all([
    called === list.name ? Promise.resolve() : listRename(list.id, called),
    drawn === (list.icon ?? undefined) && colour === (list.color ?? undefined)
      ? Promise.resolve()
      : listLook(list.id, drawn, colour),
  ]);

export const dropList = (list: List): Promise<boolean> =>
  sure(fill("listDropSure", list.name)).then((yes) =>
    yes ? listDrop(list.id).then(() => true) : false,
  );

export const listChoices = (list: List, held: number, h: ListHands): Choice[] => [
  { key: "edit", icon: "✎", label: t("listEdit"), onPick: () => h.edit(list) },
  {
    key: "archive",
    icon: "▣",
    label: t("listArchive"),
    off: held > 0,
    apart: true,
    onPick: () =>
      listArchive(list.id)
        .then(() => h.gone(list.id))
        .catch(h.fail),
  },
  {
    key: "drop",
    icon: "✕",
    label: t("listDrop"),
    danger: true,
    onPick: () =>
      dropList(list)
        .then((dropped) => dropped && h.gone(list.id))
        .catch(h.fail),
  },
];
