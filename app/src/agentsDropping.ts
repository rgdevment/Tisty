import type { Reach } from "./agentsSync";
import { sure } from "./sure";

export type DropAnswer = "away" | "tisty" | null;

export interface DropAsked {
  /** What the plain question says: what goes with it, its languages and pages, and that it cannot be undone. */
  said: string;
  reach: Reach;
}

type Asking = (asked: DropAsked) => Promise<DropAnswer>;

let asking: Asking | null = null;

export const asksDrop = (one: Asking | null): void => {
  asking = one;
};

// With no window to ask in, the plain question deletes from Tisty alone and touches no agent's file.
export const dropAsked = (asked: DropAsked): Promise<DropAnswer> =>
  asking ? asking(asked) : sure(asked.said).then((yes) => (yes ? "tisty" : null));
