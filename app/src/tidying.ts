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

export const dropWord = (reason?: string | null): Word =>
  reason === "duplicate_of" ? "flagDropCopy" : "flagDropIt";

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

// A page goes with the document that holds it, so asking for it again would find nothing.
export const goingWith = (all: Filed[], docs: Filed[]) => {
  const going = all.filter((one) => !all.some((up) => up.id === one.pageOf));
  const known = new Map([...docs, ...all].map((one) => [one.id, one]));
  const pages = [...known.values()].filter((one) => going.some((doc) => doc.id === one.pageOf));
  return { going, pages };
};

const withPages = (docs: number, pages: number): Word =>
  docs === 1
    ? pages === 1
      ? "flagDropOnePageSure"
      : "flagDropOnePagesSure"
    : pages === 1
      ? "flagDropAllPageSure"
      : "flagDropAllPagesSure";

export const dropAllAsked = (going: Filed[], pages: Filed[]): string => {
  const named = going
    .map((doc) =>
      [doc, ...pages.filter((one) => one.pageOf === doc.id)]
        .map((one) => `${one === doc ? "" : "    "}${listOf([one])}`)
        .join("\n"),
    )
    .join("\n");
  const many = String(going.length);
  return pages.length
    ? `${fill(withPages(going.length, pages.length), many, String(pages.length))}\n\n${named}`
    : fill("flagDropAllSure", many, named);
};

export const compactedBy = (doc: Filed, last = true): string =>
  doc.compacted
    ? fill(
        last ? "docCompactedBy" : "docCompactedSince",
        clientNamed(doc.compacted.via) ?? t("anAssistant"),
        dayOf(doc.compacted.at),
      )
    : "";
