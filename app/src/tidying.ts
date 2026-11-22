import type { Filed, Marked } from "./core";
import { dayOf } from "./format";
import { fill, t, type Word } from "./locales";
import { droppedTogether, tonguesNamed } from "./tongues";
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

export type Going = { doc: Filed; tongues: Filed[]; pages: Filed[] };

// A page goes with the document that holds it, so asking for it again would find nothing.
export const goingWith = (all: Filed[], docs: Filed[]): Going[] => {
  const known = [...new Map([...docs, ...all].map((one) => [one.id, one])).values()];
  const loose = all.filter((one) => !all.some((up) => up.id === one.pageOf));
  const taken = new Set<string>();
  const going: Going[] = [];
  for (const doc of [...loose.filter((one) => !one.pageOf), ...loose.filter((one) => one.pageOf)]) {
    if (taken.has(doc.id)) continue;
    const tongues = droppedTogether(known, doc);
    const pages = known.filter((one) => tongues.some((tongue) => tongue.id === one.pageOf));
    going.push({ doc, tongues, pages });
    for (const one of [...tongues, ...pages]) taken.add(one.id);
  }
  return going;
};

export const filesGoing = (going: Going[]): string[] =>
  going.flatMap(({ tongues, pages }) => [...tongues, ...pages].map((one) => one.file));

const withPages = (docs: number, pages: number): Word =>
  docs === 1
    ? pages === 1
      ? "flagDropOnePageSure"
      : "flagDropOnePagesSure"
    : pages === 1
      ? "flagDropAllPageSure"
      : "flagDropAllPagesSure";

const namedGoing = ({ doc, tongues }: Going): string =>
  tongues.length > 1 ? `${listOf([doc])} (${tonguesNamed(tongues)})` : listOf([doc]);

export const dropAllAsked = (going: Going[]): string => {
  const named = going
    .map((one) => [namedGoing(one), ...one.pages.map((page) => `    ${listOf([page])}`)].join("\n"))
    .join("\n");
  const many = String(going.length);
  const pages = going.reduce((all, one) => all + one.pages.length, 0);
  return pages
    ? `${fill(withPages(going.length, pages), many, String(pages))}\n\n${named}`
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
