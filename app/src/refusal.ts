import { noteTrouble } from "./core";
import { fill, t } from "./locales";

export interface Refusal {
  code: string;
  name?: string;
}

const KNOWN = [
  "updateBusy",
  "updateNotHere",
  "updateGone",
  "updateFailed",
  "untitled",
  "noSuchList",
  "ambiguousList",
  "badTag",
  "notATaskId",
  "onlyArchivedGoes",
  "notAClosing",
  "notAListId",
  "pastEnd",
  "manyLists",
  "notAStepId",
  "notADate",
  "notAPriority",
  "notAnEntry",
  "emptyStep",
  "emptyEntry",
  "pastDeadline",
  "pastReminder",
  "cannotRead",
  "cannotOpen",
  "noRemote",
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
  "sharedIsTheBackup",
  "otherStore",
  "syncNewer",
  "cannotWrite",
  "attachmentTooBig",
  "attachmentTooBigHere",
  "textTooLong",
  "documentTooBig",
  "documentTooLong",
  "archivedList",
  "restoreFailed",
  "stillCarrying",
  "sandboxCannotJoin",
  "notThisMachine",
  "stillReferenced",
  "internal",
  "internalNamed",
  "noSuchFolder",
  "noSuchDoc",
  "noSuchIcon",
  "noSuchColour",
  "tooDeep",
  "intoItself",
  "notACadence",
  "noClipboard",
] as const;

type Known = (typeof KNOWN)[number];

const isKnown = (code: string): code is Known => (KNOWN as readonly string[]).includes(code);

export function saidPlainly(problem: unknown): string {
  const refusal = problem as Refusal | undefined;
  if (!refusal || typeof refusal.code !== "string") {
    return technical(String(problem));
  }
  noteTrouble(refusal.code).catch(() => {});
  if (!isKnown(refusal.code)) {
    return technical(refusal.name ?? refusal.code);
  }
  return refusal.name ? fill(refusal.code, refusal.name) : t(refusal.code);
}

const technical = (raw: string): string => `${t("internal")} — ${raw}`;
