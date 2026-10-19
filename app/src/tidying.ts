import type { Filed, Marked } from "./core";
import { dayOf } from "./format";
import { fill, t, type Word } from "./locales";
import { clientNamed } from "./who";

const POINTING: Record<string, Word> = {
  duplicate_of: "flagCopyOf",
  moved: "flagMovedTo",
  superseded: "flagReplacedBy",
};

const PLAIN: Record<string, Word> = {
  wrong: "flagWrong",
  obsolete: "flagObsolete",
  scratch: "flagScratch",
};

export const pointedWord = (reason?: string | null): Word | undefined =>
  reason ? POINTING[reason] : undefined;

export const plainWord = (reason?: string | null): Word | undefined =>
  reason ? PLAIN[reason] : undefined;

export const dropWord = (reason?: string | null): Word => {
  if (reason === "wrong" || reason === "obsolete") return "flagDropIt";
  return reason && (reason in POINTING || reason === "scratch") ? "flagDropCopy" : "deleteIt";
};

const ULID = /^[0-9A-HJKMNP-TV-Z]{26}$/i;

export const looksLikeTask = (instead: string): boolean => ULID.test(instead);

const same = (one: Marked, other: Marked): boolean =>
  one.at === other.at &&
  one.said === other.said &&
  (one.reason ?? null) === (other.reason ?? null) &&
  (one.instead ?? null) === (other.instead ?? null);

export const alike = (known: Filed[], own: Filed): Filed[] => {
  const mark = own.flagged;
  if (!mark) return [];
  return known.filter((one) => !one.away && one.flagged && same(one.flagged, mark));
};

export const listOf = (docs: Filed[]): string =>
  docs.map((one) => `· ${one.title || t("untitledDoc")}`).join("\n");

export const compactedBy = (doc: Filed): string =>
  doc.compacted
    ? fill(
        "docCompactedBy",
        clientNamed(doc.compacted.via) ?? t("anAssistant"),
        dayOf(doc.compacted.at),
      )
    : "";
