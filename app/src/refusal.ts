import { Refused, sayIn } from "tisty-diagrams/read";
import { noteTrouble } from "./core";
import { budgetResets, weigh } from "./format";
import { fill, locale, t } from "./locales";

export interface Refusal {
  code: string;
  name?: string;
  print?: string;
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
  "ideaTooLong",
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
  "helpPicturesAway",
  "restoredApart",
  "syncNewer",
  "syncShape",
  "syncNeeds",
  "syncUnshaped",
  "cloudSealed",
  "cloudSealing",
  "cloudSealingHere",
  "cloudSealingStopped",
  "sealingWiping",
  "sealingBusy",
  "cloudUnsealed",
  "cloudOtherPhrase",
  "otherPerson",
  "importedFromTisty",
  "pageOfAnother",
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
  "parcelAsksIfMine",
  "noRoom",
  "nothingToCarry",
  "stillPacking",
  "aliasTooLong",
  "nameTooLong",
  "nameUnfit",
  "tooBig",
  "deleteRefused",
  "alreadyKept",
  "shedAlready",
  "stillKept",
  "noSuchIcon",
  "noSuchColour",
  "tooDeep",
  "pageOfPage",
  "tongueTaken",
  "notOneOwnDoc",
  "onlyADocumentShared",
  "noIdentityToSign",
  "cannotSeal",
  "cannotFinish",
  "folioUnreached",
  "folioGarbled",
  "folioFull",
  "folioLanguagesFull",
  "folioReviewed",
  "folioLanguageChanged",
  "folioTooBig",
  "folioWait",
  "folioUploadsOff",
  "folioClosed",
  "folioPaused",
  "folioBlocked",
  "folioBehind",
  "folioOutdated",
  "folioRevoked",
  "folioGone",
  "folioRefused",
  "folioChanged",
  "folioUnopened",
  "folioAppealed",
  "folioTermsNotPublished",
  "folioTermsUnmatched",
  "folioTermsUpdating",
  "folioKeyBlocked",
  "folioOwnerKeyBlocked",
  "folioEditTakenBack",
  "folioEditLinkPending",
  "folioEditLinkMismatch",
  "folioNewerVersionHere",
  "folioRoomFull",
  "folioOwnerRoomFull",
  "folioRoomFullAll",
  "folioRoomFullOf",
  "folioOwnerRoomFullOf",
  "folioOwnerRoomFullAll",
  "folioFullOf",
  "folioLanguagesFullOf",
  "folioQuota",
  "folioSharesToday",
  "folioTermsToAccept",
  "awayNotShared",
  "tongueUnsaid",
  "notALanguage",
  "pageHasNoTongue",
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
  "diagramMoved",
  "diagramGone",
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
  "settingsUnborn",
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

const WEIGHED = ["folioRoomFull", "folioOwnerRoomFull"];
const MINUTE_MS = 60_000;
const FIGURED = ["folioRoomFullOf", "folioOwnerRoomFullOf"];

// Folio hands what the key holds, what arrives and what fits, in that order.
const figured = (code: Parameters<typeof t>[0], name: string): string | null => {
  const [held, arriving, most] = name.split(" ").map(Number);
  if (![held, arriving, most].every(Number.isFinite)) return null;
  return t(code)
    .replace("{held}", roomOf(held))
    .replace("{arriving}", roomOf(arriving))
    .replace("{most}", roomOf(most));
};

// Folio says when, in milliseconds; a person reads how long, or at what time.
const minutesTo = (until: string) =>
  String(Math.max(1, Math.ceil((Number(until) - Date.now()) / MINUTE_MS)));

const clockAt = (until: string) =>
  new Date(Number(until)).toLocaleTimeString(locale(), { hour: "2-digit", minute: "2-digit" });
const MEGABYTE = 1_000_000;

// Folio's room is a round figure, and «25.0 MB» would read like something measured.
const roomOf = (bytes: number) =>
  bytes > 0 && bytes % MEGABYTE === 0 ? `${bytes / MEGABYTE} MB` : weigh(bytes);

export function plainly(problem: unknown): string {
  if (problem instanceof Refused) return sayIn(locale())(problem.code);
  const refusal = problem as Refusal | undefined;
  if (!refusal || typeof refusal.code !== "string") {
    return technical(String(problem));
  }
  if (!isKnown(refusal.code)) {
    return technical(refusal.name ?? refusal.code);
  }
  if (refusal.code === "cloudSpent") return fill(refusal.code, refusal.name ?? "", budgetResets());
  if (FIGURED.includes(refusal.code) && refusal.name) {
    const said = figured(refusal.code, refusal.name);
    if (said) return said;
  }
  if (WEIGHED.includes(refusal.code) && refusal.name) {
    return fill(refusal.code, roomOf(Number(refusal.name)));
  }
  if (refusal.code === "folioWait" && refusal.name)
    return fill(refusal.code, minutesTo(refusal.name));
  if (refusal.code === "folioQuota" && refusal.name)
    return fill(refusal.code, clockAt(refusal.name));
  return refusal.name ? fill(refusal.code, refusal.name) : t(refusal.code);
}

const technical = (raw: string): string => `${t("internal")} — ${raw}`;
