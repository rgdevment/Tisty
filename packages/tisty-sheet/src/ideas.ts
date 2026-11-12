import { CONTRACTS } from "@tisty/contracts";

type Style = { values: { callouts: { idea: { hues: string[] } } } };

export const IDEA_HUES: readonly string[] = (CONTRACTS.style as Style).values.callouts.idea.hues;

const STAMPED = /^\d{4}-\d{2}-\d{2}(?: \d{2}:\d{2})?$/;

export interface IdeaHead {
  when?: string;
  hue?: string;
  side?: "left" | "right";
}

export const ideaParts = (words: string): IdeaHead => {
  const head: IdeaHead = {};
  for (const part of words.split(" · ").map((one) => one.trim())) {
    const lower = part.toLowerCase();
    if (IDEA_HUES.includes(lower)) head.hue = lower;
    else if (lower === "left" || lower === "right") head.side = lower;
    else if (!head.when && STAMPED.test(part)) head.when = part;
  }
  return head;
};

export const saysAny = (head: IdeaHead): boolean => Boolean(head.when || head.hue || head.side);
