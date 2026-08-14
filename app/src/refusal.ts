import { noteTrouble } from "./core";
import { fill, t } from "./locales";

export interface Refusal {
  code: string;
  name?: string;
}

const KNOWN = [
  "untitled",
  "noSuchList",
  "ambiguousList",
  "badTag",
  "notATaskId",
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
  "syncUnreadable",
  "syncRefused",
  "syncBroke",
  "wouldMerge",
  "remoteInsideStore",
  "sharedIsTheBackup",
  "otherStore",
  "cannotWrite",
  "attachmentTooBig",
  "documentTooBig",
  "restoreFailed",
  "stillCarrying",
  "sandboxCannotMerge",
  "internal",
  "internalNamed",
  "noSuchFolder",
  "noSuchDoc",
  "noSuchIcon",
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
