import { t } from "./locales";
import { stored } from "./stored";
import type { Choice } from "./ui/Menu";

export const DIARY = "diary";
export const BRAINSTORM = "brainstorm";
export const AGENTS = "agents";

// Kept on this machine only: hiding is a way of looking, and another computer may want it shown.
const kept = stored<{ keys: string[] }>("tisty.hidden", (said) => ({
  keys: Array.isArray(said.keys) ? said.keys.filter((one) => typeof one === "string") : [],
}));

export const useHidden = (): string[] => kept.use().keys;

export const isHidden = (key: string): boolean => kept.current().keys.includes(key);

export const hide = (key: string) => {
  if (!isHidden(key)) kept.put({ keys: [...kept.current().keys, key] });
};

export const show = (...keys: string[]) =>
  kept.put({ keys: kept.current().keys.filter((one) => !keys.includes(one)) });

export const showAll = () => kept.put({ keys: [] });

// AGENTS stands in the sidebar even without its folder, and a folder made again gets a new id.
export const agentsKeys = (root?: string | null): string[] => (root ? [AGENTS, root] : [AGENTS]);

export const agentsHidden = (hidden: readonly string[], root?: string | null): boolean =>
  agentsKeys(root).some((key) => hidden.includes(key));

export const hideChoice = (keys: string | string[], off = false): Choice => {
  const all = typeof keys === "string" ? [keys] : keys;
  return all.some(isHidden)
    ? {
        key: "hide",
        icon: "◉",
        label: t("showInSidebar"),
        off,
        apart: true,
        onPick: () => show(...all),
      }
    : {
        key: "hide",
        icon: "◌",
        label: t("hideFromSidebar"),
        off,
        apart: true,
        onPick: () => hide(all[0]),
      };
};
