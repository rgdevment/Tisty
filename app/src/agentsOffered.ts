import type { Wired } from "./core";
import { stored } from "./stored";

export const SKILLED: Record<string, string> = {
  "claude-code": "claude",
  cursor: "cursor",
  codex: "codex",
  "gemini-cli": "gemini",
  copilot: "copilot",
};

export const SKILLS_DECLINED = "tisty-skills";

export const offeredHere = stored<string[]>("tisty.wiring.offered", (said) =>
  Array.isArray(said) ? said.filter((one) => typeof one === "string") : [],
);

export const offered = (): string[] => offeredHere.current();

export const markOffered = (ids: readonly string[]) => {
  const next = [...new Set([...offered(), ...ids])];
  if (next.length !== offered().length) offeredHere.put(next);
};

export const offers = (one: Wired): boolean => !!one.hooks || one.id in SKILLED;
