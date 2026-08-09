import type { List, View } from "./core";
import { fill, t } from "./locales";

export type Named = "search" | "inbox" | "today" | "upcoming" | "tags" | "archive";

export interface Chosen {
  named?: Named;
  list?: string;
  tags?: string[];
  folded?: boolean;
}

export function asView(chosen: Chosen): View {
  if (chosen.list) return { list: chosen.list };
  if (chosen.tags?.length) return { tags: chosen.tags, everything: true };

  switch (chosen.named) {
    case "inbox":
      return { inbox: true };
    case "today":
      return { window: "today" };
    case "upcoming":
      return { window: "upcoming" };
    case "archive":
      return { archive: true, hidden: chosen.folded };
    case "tags":
      return { tagged: true, everything: true };
    default:
      return {};
  }
}

export function title(chosen: Chosen, lists: List[]): string {
  if (chosen.list) {
    return lists.find((list) => list.id === chosen.list)?.name ?? "";
  }
  if (chosen.tags?.length) {
    return chosen.tags.map((tag) => `#${tag}`).join(" ");
  }
  return t(chosen.named ?? "today");
}

/** Archive and search have nothing to add to; upcoming has no day to pick. */
export function accepts(chosen: Chosen): boolean {
  if (chosen.named === "archive" || chosen.named === "search") return false;
  if (chosen.named === "upcoming") return false;
  if (chosen.named === "tags") return (chosen.tags?.length ?? 0) > 0;
  return true;
}

/** Says where the task will land, because it lands where you are looking. */
export function invite(chosen: Chosen, lists: List[]): string {
  if (chosen.list) {
    const name = lists.find((list) => list.id === chosen.list)?.name;
    return name ? fill("addToList", name) : t("addTask");
  }
  if (chosen.tags?.length) {
    return fill("addWithTag", chosen.tags.map((tag) => `#${tag}`).join(" "));
  }
  if (chosen.named === "inbox") return t("addToInbox");
  if (chosen.named === "today") return t("addForToday");
  return t("addTask");
}
