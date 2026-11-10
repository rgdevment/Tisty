import { stored } from "./stored";

export type Lean = "tasks" | "docs";

const kept = stored<{ on: Lean }>("tisty.lean", (said) => ({
  on: said.on === "tasks" ? "tasks" : "docs",
}));

export const useLean = (): Lean => kept.use().on;

export const lean = (on: Lean) => kept.put({ on });
