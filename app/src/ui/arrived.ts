import type { Came } from "../core";
import { t, type Word } from "../locales";
import { dated } from "./Rows";

const HOW: Record<string, Word> = {
  window: "cameWindow",
  assistant: "cameAssistant",
  imported: "cameImported",
  parcel: "cameParcel",
  guide: "cameGuide",
};

// One kept before 1.33 never said how, so it tells only where and when.
export const cameWords = (came?: Came | null): string | null => {
  if (!came) return null;
  const how = came.how ? HOW[came.how] : undefined;
  return [how && t(how), came.mine ? t("cameHere") : came.machine, dated(came.when)]
    .filter(Boolean)
    .join(" · ");
};
