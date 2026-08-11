import type { List, View } from "./core";
import { fill, t } from "./locales";

export type Named = "search" | "inbox" | "today" | "upcoming" | "tags" | "archive" | "keeping";

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
  if (chosen.named === "keeping") return false;
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

/**
 * What an empty screen should say. One line for six different situations told
 * the reader nothing at the only moment they were going to read.
 */
export function nothing(chosen: Chosen, searching: boolean): string {
  if (searching) return t("noHits");
  if (chosen.named === "search") return t("searchInvite");
  if (chosen.named === "archive") return t("archiveEmpty");
  if (chosen.named === "tags") return t("noTagsYet");
  if (chosen.list || chosen.tags?.length) return t("listEmpty");
  if (chosen.named === "upcoming") return t("upcomingEmpty");
  if (chosen.named === "inbox") return t("inboxEmpty");
  return t("todayEmpty");
}
