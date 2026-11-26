import { MEASURES, measureFor, measureNow } from "./breadth";
import type { Filed } from "./core";
import { t } from "./locales";
import type { Choice } from "./ui/Menu";

const LABEL = {
  narrow: "measureNarrow",
  normal: "measureNormal",
  wide: "measureWide",
  full: "measureFull",
} as const;

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
        label: t(LABEL[one]),
        on: one === now,
        onPick: () => measureFor(doc.id, one),
      })),
    },
  };
};
