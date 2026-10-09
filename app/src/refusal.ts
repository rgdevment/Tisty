import { noteTrouble } from "./core";
import { fill, t } from "./locales";

export interface Refusal {
  code: string;
  name?: string;
}

const KNOWN = [
  "updateBusy",
  "updateElsewhere",
  "updateNotHere",
  "updateGone",
  "updateMoved",
  "updateStopped",
  "updateFailed",
  "updateUnanswered",
  "untitled",
  "noSuchList",
  "ambiguousList",
  "badTag",
  "notATaskId",
  "onlyArchivedGoes",
  "listStillOpen",
  "markEntry",
  "notAClosing",
  "notATheme",
  "storyStays",
  "routineStays",
  "partsStay",
  "partRepeats",
  "wholeRepeats",
  "partOfItself",
  "partOfAPart",
  "wholeIsNoPart",
  "wholeClosed",
  "onlyClosedConverts",
  "routineReadsAsRoutine",
  "notAReading",
  "onlyOpenOpens",
  "alreadyTheirs",
  "notAListId",
  "pastEnd",
  "manyLists",
  "notAStepId",
  "notADate",
  "notAPriority",
  "notAnEntry",
  "emptyStep",
  "stepTooLong",
  "emptyEntry",
  "pastDeadline",
  "pastReminder",
  "cannotRead",
  "cannotOpen",
  "noRemote",
  "syncLater",
  "syncLaterToLeave",
  "settingsUnreadable",
  "noMeetingPlace",
  "emptiedPlace",
  "syncUnreadable",
  "syncRefused",
  "syncBroke",
  "wouldReset",
  "sameName",
  "noBase",
  "cannotWeave",
  "movedUnderfoot",
  "notAllowed",
  "remoteInsideStore",
  "otherStore",
  "restoredApart",
  "syncNewer",
  "syncShape",
  "syncUnshaped",
  "storeNewer",
  "cannotWrite",
  "attachmentTooBig",
  "attachmentTooBigHere",
  "textTooLong",
  "widgetTooBig",
  "pageTooBig",
  "documentTooBig",
  "documentTooLong",
  "archivedList",
  "restoreFailed",
  "stillCarrying",
  "sandboxCannotJoin",
  "notThisMachine",
  "keyMoved",
  "keyNotConfirmed",
  "stillReferenced",
  "internal",
  "internalNamed",
  "noSuchFolder",
  "noSuchDoc",
  "notAParcel",
  "parcelNewer",
  "parcelLocked",
  "wrongNumber",
  "parcelTorn",
  "noRoom",
  "nothingToCarry",
  "stillPacking",
  "aliasTooLong",
  "tooBig",
  "deleteRefused",
  "alreadyKept",
  "shedAlready",
  "stillKept",
  "noSuchIcon",
  "noSuchColour",
  "tooDeep",
  "pageOfPage",
  "pageStaysPut",
  "holdsPages",
  "pageOfAway",
  "awayStaysAway",
  "folderNameSlash",
  "folderNameTooLong",
  "documentBeingWritten",
  "documentLocked",
  "documentAway",
  "folderAway",
  "folderAwayHolds",
  "folderIsAway",
  "pageIsAway",
  "pageOfLocked",
  "lockedStaysPut",
  "lockIsTheDocs",
  "documentMoved",
  "nothingKeptBeside",
  "comingDown",
  "docComing",
  "sharedAway",
  "heldAway",
  "attachmentTorn",
  "sharedAwayToLeave",
  "intoItself",
  "notACadence",
  "noClipboard",
  "noSuchAgent",
  "settingsPuzzling",
] as const;

type Known = (typeof KNOWN)[number];

const isKnown = (code: string): code is Known => (KNOWN as readonly string[]).includes(code);

const BEHIND = [
  "storeNewer",
  "syncNewer",
  "syncShape",
  "syncLater",
  "syncLaterToLeave",
  "settingsUnreadable",
];

export const folderAhead = (problem: unknown): { name?: string } | null => {
  const refusal = problem as Refusal | undefined;
  return refusal?.code === "syncNewer" || refusal?.code === "syncShape"
    ? { name: refusal.name }
    : null;
};

/** The offer the person clicked is off the feed: what is offered now has to be looked up again. */
export const offerMoved = (problem: unknown): boolean =>
  ["updateGone", "updateMoved"].includes((problem as Refusal | undefined)?.code ?? "");

let noticing: ((behind: boolean) => void) | null = null;

export const noticeBehind = (fn: ((behind: boolean) => void) | null) => {
  noticing = fn;
};

export function saidPlainly(problem: unknown): string {
  const refusal = problem as Refusal | undefined;
  if (refusal && typeof refusal.code === "string") {
    noteTrouble(refusal.code, refusal.name).catch(() => {});
    noticing?.(BEHIND.includes(refusal.code));
  }
  return plainly(problem);
}

export function plainly(problem: unknown): string {
  const refusal = problem as Refusal | undefined;
  if (!refusal || typeof refusal.code !== "string") {
    return technical(String(problem));
  }
  if (!isKnown(refusal.code)) {
    return technical(refusal.name ?? refusal.code);
  }
  return refusal.name ? fill(refusal.code, refusal.name) : t(refusal.code);
}

const technical = (raw: string): string => `${t("internal")} — ${raw}`;
