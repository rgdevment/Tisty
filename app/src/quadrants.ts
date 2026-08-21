import type { Priority } from "./core";
import { t } from "./locales";

export const QUADRANTS = ["do", "decide", "delegate", "wont"] as const;

export const PICKABLE = ["do", "decide", "delegate"] as const;

const SAID = {
  do: "quadDo",
  decide: "quadDecide",
  delegate: "quadDelegate",
  wont: "quadWont",
  unset: "noPriority",
} as const;

const TINT = {
  do: "text-urgent",
  decide: "text-high",
  delegate: "text-accent",
  wont: "text-faint",
  unset: "text-faint",
} as const;

const EDGE = {
  do: "border-urgent",
  decide: "border-high",
  delegate: "border-accent",
  wont: "border-faint",
  unset: "border-faint",
} as const;

export const placed = (p: Priority): boolean => p !== "unset";

export const said = (p: Priority): string => t(SAID[p]);

export const tint = (p: Priority): string => TINT[p];

export const edge = (p: Priority): string => EDGE[p];
