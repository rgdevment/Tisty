import type { List, View } from "./core";
import { t } from "./locales";

export type Named = "inbox" | "today" | "upcoming" | "tags" | "archive";

export interface Chosen {
  named?: Named;
  list?: string;
}

export function asView(chosen: Chosen): View {
  if (chosen.list) return { list: chosen.list };

  switch (chosen.named) {
    case "inbox":
      return { inbox: true };
    case "today":
      return { window: "today" };
    case "upcoming":
      return { window: "upcoming" };
    case "archive":
      return { archive: true };
    default:
      return {};
  }
}

export function title(chosen: Chosen, lists: List[]): string {
  if (chosen.list) {
    return lists.find((list) => list.id === chosen.list)?.name ?? "";
  }
  return t(chosen.named ?? "today");
}
