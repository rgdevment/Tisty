import type { Say } from "tisty-editor/read";
import { fill, t } from "../locales";

// The sheet says its words in Tisty's, with what each one names filled in.
export const SAY: Say = (word, ...filled) =>
  filled.length ? fill(word, filled[0], filled[1]) : t(word);
