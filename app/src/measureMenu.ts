import { MEASURES, measureFor, measureNow } from "./breadth";
import type { Filed } from "./core";
import { t } from "./locales";
import type { Place } from "./places";
import type { Choice } from "./ui/Menu";

export const MEASURE_WORDS = {
  narrow: "measureNarrow",
  normal: "measureNormal",
  wide: "measureWide",
  full: "measureFull",
} as const;

// Ideas and the diary have their own page and an agent's file its card: only a plain document's text widens.
export const widens = (place: Place): boolean => place === "plain";

export const measureChoice = (doc: Filed): Choice => {
  const now = measureNow(doc.id);
  return {
    key: "measure",
    icon: "↔",
    label: t("measure"),
    into: {
      label: t("measure"),
      choices: MEASURES.map((one) => ({
        key: one,
        label: t(MEASURE_WORDS[one]),
        on: one === now,
        onPick: () => measureFor(doc.id, one),
      })),
    },
  };
};
