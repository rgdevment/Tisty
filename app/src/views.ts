import type { List, View } from "./core";
import { fill, t } from "./locales";

export type Named = "search" | "inbox" | "tasks" | "tags" | "archive" | "keeping";

/// «Hoy» and «Próximo» were two names for one thing: a date filter with a fixed
/// seat in the sidebar. They are the same view now, and this is what it filters
/// by — opening on `today`, because «all» is forty rows the moment you arrive.
export type Slice = "today" | "upcoming" | "undated" | "all";

export const SLICES: Slice[] = ["today", "upcoming", "undated", "all"];

export interface Chosen {
  named?: Named;
  list?: string;
  tags?: string[];
  folded?: boolean;
  slice?: Slice;
}

export function asView(chosen: Chosen): View {
  if (chosen.list) return { list: chosen.list };
  if (chosen.tags?.length) return { tags: chosen.tags, everything: true };

  switch (chosen.named) {
    case "inbox":
      return { inbox: true };
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

/// `all` asks for no window at all, which is what «everything still open» is.
function sliced(slice: Slice = "today"): View {
  switch (slice) {
    case "today":
      return { window: "today" };
    case "upcoming":
      return { window: "upcoming" };
    case "undated":
      return { window: "undated" };
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
  // Nothing to add TO: a task captured here would land on a day this slice
  // does not show, and vanish the moment it was written.
  if (chosen.named === "tasks" && chosen.slice && chosen.slice !== "today") return false;
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
  if (chosen.named === "tasks" && (chosen.slice ?? "today") === "today") return t("addForToday");
  return t("addTask");
}

/**
 * What an empty screen should say. One line for six different situations told
 * the reader nothing at the only moment they were going to read.
 */
export function nothing(chosen: Chosen, searching: boolean): string {
  // The archive hides the scope chips, so «widen the scope above» would point
  // at something that is not on screen.
  if (searching) return t(chosen.named === "archive" ? "noHitsHere" : "noHits");
  if (chosen.named === "search") return t("searchInvite");
  if (chosen.named === "archive") return t("archiveEmpty");
  // Before the bare tag view: choosing tags sets `named` AND `tags`, and the
  // screen said «no tags yet» with the chosen tags drawn right above it.
  if (chosen.list || chosen.tags?.length) return t("listEmpty");
  if (chosen.named === "tags") return t("noTagsYet");
  if (chosen.named === "inbox") return t("inboxEmpty");
  if (chosen.slice === "upcoming") return t("upcomingEmpty");
  if (chosen.slice === "undated") return t("undatedEmpty");
  if (chosen.slice === "all") return t("allEmpty");
  return t("todayEmpty");
}
