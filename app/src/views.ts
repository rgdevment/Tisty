import { useCallback, useState } from "react";
import type { Axis } from "./archive";
import type { List, Reading, View } from "./core";
import { ahead, every, repeating, turnOf, unlisted } from "./examples";
import { fill, t } from "./locales";
import type { RowId, Shelf } from "./settingsRows";

export type Named =
  | "search"
  | "tasks"
  | "quadrants"
  | "spread"
  | "lists"
  | "docs"
  | "tags"
  | "archive"
  | "keeping"
  | "weekly"
  | "help"
  | "brainsty";

export type Tab =
  | "summary"
  | "window"
  | "notices"
  | "writing"
  | "sync"
  | "security"
  | "copies"
  | "storage"
  | "agents"
  | "health"
  | "about";

export type Heading = "pick" | "cloud" | "folder";

export type Lean = "tasks" | "docs";

export type Slice = "today" | "upcoming" | "repeating" | "all" | "overdue" | "late" | "said";

export const SLICES: Slice[] = ["today", "upcoming", "repeating", "all"];

export const STANDINGS: Slice[] = ["overdue", "late"];

export const LAYERS: Reading[] = ["story", "routine", "trace"];

export const layerWord = (layer: Reading) =>
  layer === "story"
    ? ("layerStories" as const)
    : layer === "routine"
      ? ("layerRoutines" as const)
      : ("layerTrace" as const);

export const layerCount = (layer: Reading) =>
  layer === "story"
    ? ("stories" as const)
    : layer === "routine"
      ? ("routines" as const)
      : ("traces" as const);

export const axisWord = (axis: Axis) =>
  axis === "time"
    ? ("axisTime" as const)
    : axis === "list"
      ? ("axisList" as const)
      : axis === "tag"
        ? ("axisTag" as const)
        : ("axisQuadrant" as const);

export interface Chosen {
  named?: Named;
  doc?: string;
  near?: string;
  besideTask?: string;
  tab?: Tab;
  shelf?: Shelf;
  row?: RowId;
  landedAt?: number;
  move?: Heading;
  movedAt?: number;
  list?: string;
  lists?: string[];
  tags?: string[];
  folded?: boolean;
  slice?: Slice;
  layer?: Reading;
  axis?: Axis;
  topic?: string;
  article?: string;
  heading?: string;
  from?: string;
}

export const A_COLUMN = 200;

export function asView(chosen: Chosen, reach: number = A_COLUMN): View {
  if (chosen.list) return { list: chosen.list, most: reach };
  if (chosen.tags?.length) return { tags: chosen.tags, everything: true, most: reach };

  switch (chosen.named) {
    case "tasks":
      return chosen.lists?.length
        ? { ...sliced(chosen.slice), lists: chosen.lists, most: reach }
        : { ...sliced(chosen.slice), most: reach };
    case "archive":
      return chosen.folded
        ? { archive: true, hidden: true, most: reach }
        : { archive: true, reading: chosen.layer ?? "story", most: reach };
    case "tags":
      return { tagged: true, everything: true, most: reach };
    case "quadrants":
      return { board: true, most: reach };
    case "spread":
      return { spread: true, most: reach };
    case "lists":
      return { soonest: true, most: reach };
    default:
      return { most: reach };
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
    case "overdue":
      return { window: "overdue" };
    case "late":
      return { window: "late" };
    case "said":
      return { said: true };
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
  if (chosen.named === "quadrants") return false;
  if (chosen.named === "keeping") return false;
  if (chosen.named === "weekly") return false;
  if (chosen.named === "help") return false;
  if (chosen.named === "brainsty") return false;

  if (chosen.named === "tags") return (chosen.tags?.length ?? 0) > 0;
  return true;
}

// An example is offered only where a capture field is there to take it.
export const offers = (chosen: Chosen, searching: boolean): boolean =>
  !searching && accepts(chosen);

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

const offered = (all: string[], at: number): string => turnOf(unlisted(all), at);

// `at`, from 0 up to 1, says whose turn it is among the examples a view may offer.
export function nothing(chosen: Chosen, searching: boolean, tracesHidden = 0, at = 0): string {
  if (searching) return t(chosen.named === "archive" ? "noHitsHere" : "noHits");
  if (chosen.named === "search") return t("searchInvite");
  if (chosen.named === "archive") {
    if (chosen.folded) return t("archiveEmpty");
    const layer = chosen.layer ?? "story";
    if (layer === "trace" && tracesHidden > 0) return fill("traceHidden", String(tracesHidden));
    return t(
      layer === "story" ? "storiesEmpty" : layer === "routine" ? "routinesEmpty" : "traceEmpty",
    );
  }
  if (chosen.list || chosen.tags?.length) return fill("listEmpty", offered(every(), at));
  if (chosen.named === "tags") return t("noTagsYet");
  if (chosen.slice === "upcoming") return fill("upcomingEmpty", offered(ahead(), at));
  if (chosen.slice === "overdue") return t("overdueEmpty");
  if (chosen.slice === "late") return t("lateEmpty");
  if (chosen.slice === "said") return t("saidEmpty");
  if (chosen.slice === "repeating") return fill("repeatingEmpty", offered(repeating(), at));
  if (chosen.lists?.length) return fill("listEmpty", offered(every(), at));
  if (chosen.slice === "all") return t("allEmpty");
  return fill("todayEmpty", offered(every(), at));
}

export function headerCount(
  chosen: Chosen,
  found: { tasks: unknown[] } | null,
  counts: Record<string, number>,
  total: number,
): number | undefined {
  if (chosen.named === "tasks") return undefined;
  if (chosen.named === "archive" && !chosen.folded && chosen.layer === "routine") {
    return counts.routines;
  }
  if (found) return found.tasks.length;
  if (chosen.named === "search") return undefined;
  return total;
}

export function useReach(chosen: Chosen) {
  const [reach, setReach] = useState(A_COLUMN);
  const [seen, setSeen] = useState(chosen);
  const further = useCallback(() => setReach((was) => was + A_COLUMN), []);
  if (seen !== chosen) {
    setSeen(chosen);
    setReach(A_COLUMN);
    return { reach: A_COLUMN, further };
  }
  return { reach, further };
}
