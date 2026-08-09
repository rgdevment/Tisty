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
  "internal",
] as const;

type Known = (typeof KNOWN)[number];

const isKnown = (code: string): code is Known => (KNOWN as readonly string[]).includes(code);

/** An unknown code shows itself rather than swallowing what went wrong. */
export function saidPlainly(problem: unknown): string {
  const refusal = problem as Refusal | undefined;
  if (!refusal || typeof refusal.code !== "string") {
    return String(problem);
  }
  if (!isKnown(refusal.code)) {
    return refusal.name ?? refusal.code;
  }
  return refusal.name ? fill(refusal.code, refusal.name) : t(refusal.code);
}
