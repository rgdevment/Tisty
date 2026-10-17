import type { Left } from "./core";
import { fill } from "./locales";

export const leftBehind = ({ missed, notes }: Left): string =>
  [
    missed > 0 ? fill("importMissed", String(missed)) : "",
    notes > 0 ? fill("importNotes", String(notes)) : "",
  ]
    .filter(Boolean)
    .join(" ");
