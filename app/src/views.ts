import type { List, View } from "./core";
import { fill, t } from "./locales";

export type Named = "search" | "tasks" | "lists" | "docs" | "tags" | "archive" | "keeping" | "aboutScreen";

export type Slice = "today" | "upcoming" | "repeating" | "all";

export const SLICES: Slice[] = ["today", "upcoming", "repeating", "all"];

export interface Chosen {
  named?: Named;
  doc?: string;
  list?: string;
  tags?: string[];
  folded?: boolean;
  slice?: Slice;
}

export function asView(chosen: Chosen): View {
  if (chosen.list) return { list: chosen.list };
  if (chosen.tags?.length) return { tags: chosen.tags, everything: true };

  switch (chosen.named) {
    case "tasks":
      return sliced(chosen.slice);
    case "archive":
      return { archive: true, hidden: chosen.folded };
    case "tags":
      return { tagged: true, everything: true };
    default:
      return {};
  }
}

function sliced(slice: Slice = "today"): View {
  switch (slice) {
    case "today":
      return { window: "today" };
    case "upcoming":
      return { window: "upcoming" };
    case "repeating":
      return { repeating: true };
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

export function accepts(chosen: Chosen): boolean {
  if (chosen.named === "archive" || chosen.named === "search") return false;
  if (chosen.named === "keeping") return false;

  if (chosen.named === "tags") return (chosen.tags?.length ?? 0) > 0;
  return true;
}

export function invite(chosen: Chosen, lists: List[]): string {
  if (chosen.list) {
    const name = lists.find((list) => list.id === chosen.list)?.name;
    return name ? fill("addToList", name) : t("addTask");
  }
  if (chosen.tags?.length) {
    return fill("addWithTag", chosen.tags.map((tag) => `#${tag}`).join(" "));
  }
  if (chosen.named === "tasks" && (chosen.slice ?? "today") === "today") return t("addForToday");
  return t("addTask");
}

export function nothing(chosen: Chosen, searching: boolean): string {
  if (searching) return t(chosen.named === "archive" ? "noHitsHere" : "noHits");
  if (chosen.named === "search") return t("searchInvite");
  if (chosen.named === "archive") return t("archiveEmpty");
  if (chosen.list || chosen.tags?.length) return t("listEmpty");
  if (chosen.named === "tags") return t("noTagsYet");
  if (chosen.slice === "upcoming") return t("upcomingEmpty");
  if (chosen.slice === "repeating") return t("repeatingEmpty");
  if (chosen.slice === "all") return t("allEmpty");
  return t("todayEmpty");
}
