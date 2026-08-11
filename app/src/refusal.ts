import { noteTrouble } from "./core";
import { fill, t } from "./locales";

/** What a rejected command sends back: a reason, not a sentence. */
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
  "syncBroke",
  "wouldMerge",
  "remoteInsideStore",
  "sharedIsTheBackup",
  "otherStore",
  "cannotWrite",
  "restoreFailed",
  "stillCarrying",
  "sandboxCannotMerge",
  "internal",
  "internalNamed",
] as const;

type Known = (typeof KNOWN)[number];

const isKnown = (code: string): code is Known => (KNOWN as readonly string[]).includes(code);

/** An unknown code shows itself rather than swallowing what went wrong. */
export function saidPlainly(problem: unknown): string {
  const refusal = problem as Refusal | undefined;
  if (!refusal || typeof refusal.code !== "string") {
    return technical(String(problem));
  }
  // The cause was written down where it happened; this is the other half of the
  // story — what the person was actually shown.
  noteTrouble(refusal.code).catch(() => {});
  if (!isKnown(refusal.code)) {
    return technical(refusal.name ?? refusal.code);
  }
  return refusal.name ? fill(refusal.code, refusal.name) : t(refusal.code);
}

/// The Rust text stays — it is what makes a report worth anything — but never
/// as the whole message: it is English, technical, and arrives at the worst
/// possible moment.
const technical = (raw: string): string => `${t("internal")} — ${raw}`;
