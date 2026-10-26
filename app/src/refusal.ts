import { noteTrouble } from "./core";
import { budgetResets } from "./format";
import { fill, t } from "./locales";

export interface Refusal {
  code: string;
  name?: string;
}

const KNOWN = [
  "updateBusy",
  "updateElsewhere",
  "updateUnconfirmed",
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
  "taskClosed",
  "icsNotACalendar",
  "icsTooBig",
  "icsTooMany",
  "icsUnreadable",
  "icsChanged",
  "onlyClosedConverts",
  "routineReadsAsRoutine",
  "notAReading",
  "onlyOpenOpens",
  "alreadyTheirs",
  "notAListId",
  "pastEnd",
  "manyLists",
  "notADate",
  "notAPriority",
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
  "syncFailed",
  "syncStuckDoc",
  "syncStuckHistory",
  "cloudSpent",
  "cloudNoAnswer",
  "cloudDropped",
  "cloudEmptied",
  "cloudFull",
  "cloudLost",
  "cloudElsewhere",
  "cloudNotNow",
  "cloudToLeave",
  "cloudNeedsMove",
  "cloudNotOffered",
  "cloudDenied",
  "davNotAnAddress",
  "davDenied",
  "davRefused",
  "davUnguarded",
  "changeUnbacked",
  "movedAway",
  "movedAlone",
  "movedToCloud",
  "moveSamePlace",
  "moveFromUnused",
  "wipeUntold",
  "moveFromUnreachable",
  "moveToUnreachable",
  "wipeChosen",
  "wipeNotHome",
  "wipeStopped",
  "wipeGoing",
  "twinsGone",
  "twinsLocked",
  "twinsStill",
  "twinsTooLong",
  "twinsEdited",
  "twinsDocTooLong",
  "twinsTaskGone",
  "forgetUntold",
  "keysTaken",
  "keysUnknown",
  "restoreCloudOff",
  "cloudNotLost",
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
  "syncNeeds",
  "syncUnshaped",
  "cloudSealed",
  "cloudSealing",
  "cloudSealingHere",
  "sealingWiping",
  "cloudUnsealed",
  "cloudOtherPhrase",
  "cloudMarkDamaged",
  "sealingNoCloud",
  "sealingPhrase",
  "sealingBehind",
  "admitNotNow",
  "admitGone",
  "admitTaken",
  "admitChanged",
  "admitWrongCode",
  "admitNotYet",
  "admitAlreadyIn",
  "admitOnList",
  "admitNoKey",
  "phraseNotThisCloud",
  "storeNewer",
  "cannotWrite",
  "attachmentTooBig",
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
  "identityExists",
  "identityNotKept",
  "phraseCount",
  "phraseWord",
  "phraseChecksum",
  "identityElsewhere",
  "identityUnread",
  "identityNone",
  "phraseOtherPerson",
  "stillReferenced",
  "retireUnread",
  "retireUnreadAny",
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
  "nameTooLong",
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
  "taskPageBehind",
  "nothingKeptBeside",
  "comingDown",
  "cloudLater",
  "cloudGone",
  "cannotDraw",
  "cannotTurn",
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
  "wslShut",
  "wslUnread",
] as const;

type Known = (typeof KNOWN)[number];

const isKnown = (code: string): code is Known => (KNOWN as readonly string[]).includes(code);

const BEHIND = [
  "storeNewer",
  "syncNewer",
  "syncShape",
  "syncNeeds",
  "syncLater",
  "syncLaterToLeave",
  "settingsUnreadable",
];

export const folderAhead = (problem: unknown): { name?: string } | null => {
  const refusal = problem as Refusal | undefined;
  if (refusal?.code === "syncNewer") return { name: refusal.name };
  return ["syncShape", "syncNeeds"].includes(refusal?.code ?? "") ? {} : null;
};

/** The offer the person clicked is off the feed: what is offered now has to be looked up again. */
export const offerMoved = (problem: unknown): boolean =>
  ["updateGone", "updateMoved"].includes((problem as Refusal | undefined)?.code ?? "");

/** The others moved away, so the way out is choosing again in Settings. */
export const leftBehind = (problem: unknown): boolean =>
  ["movedAway", "movedAlone", "movedToCloud"].includes(
    (problem as Refusal | undefined)?.code ?? "",
  );

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
  if (refusal.code === "cloudSpent") return fill(refusal.code, refusal.name ?? "", budgetResets());
  return refusal.name ? fill(refusal.code, refusal.name) : t(refusal.code);
}

const technical = (raw: string): string => `${t("internal")} — ${raw}`;
