import { t } from "./locales";
import { stored } from "./stored";
import type { Choice } from "./ui/Menu";

export const DIARY = "diary";
export const BRAINSTORM = "brainstorm";

// Kept on this machine only: hiding is a way of looking, and another computer may want it shown.
const kept = stored<{ keys: string[] }>("tisty.hidden", (said) => ({
  keys: Array.isArray(said.keys) ? said.keys.filter((one) => typeof one === "string") : [],
}));

export const useHidden = (): string[] => kept.use().keys;

export const isHidden = (key: string): boolean => kept.current().keys.includes(key);

export const hide = (key: string) => {
  if (!isHidden(key)) kept.put({ keys: [...kept.current().keys, key] });
};

export const show = (key: string) =>
  kept.put({ keys: kept.current().keys.filter((one) => one !== key) });

export const showAll = () => kept.put({ keys: [] });

export const hideChoice = (key: string, off = false): Choice =>
  isHidden(key)
    ? {
        key: "hide",
        icon: "◉",
        label: t("showInSidebar"),
        off,
        apart: true,
        onPick: () => show(key),
      }
    : {
        key: "hide",
        icon: "◌",
        label: t("hideFromSidebar"),
        off,
        apart: true,
        onPick: () => hide(key),
      };
