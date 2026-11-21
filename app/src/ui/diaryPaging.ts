import type { DayAt } from "../diary";
import { modKey } from "./macKeys";

export type DayWent =
  | { kind: "day"; at: DayAt }
  | { kind: "dayKeep" }
  | { kind: "dayCancel" }
  | { kind: "unstick"; at: DayAt; note: number };

export const NOTE = 'blockquote.callout[data-callout="idea"]';

const dayOf = (one: Element | null | undefined): DayAt | null => {
  const entry = one?.closest<HTMLElement>(".diary-entry[data-day][data-nth]");
  return entry ? { day: Number(entry.dataset.day), nth: Number(entry.dataset.nth) } : null;
};

// A day opens from its words; its date, its notes and what was closed on it keep their own press.
export const dayTouched = (target: Element): DayWent | null | undefined => {
  const at = dayOf(target);
  if (!at) return undefined;
  const elsewhere = `.diary-closed, .diary-box, .diary-box-bar, ${NOTE}`;
  return target.closest(".diary-words") && !target.closest(elsewhere) ? { kind: "day", at } : null;
};

export const dayPressed = (asked: HTMLElement): DayWent | null | undefined => {
  const kind = asked.dataset.asked;
  if (kind === "dayKeep" || kind === "dayCancel") return { kind };
  if (kind !== "unstick") return undefined;
  const note = asked.closest(NOTE);
  const words = note?.parentElement;
  const at = dayOf(note);
  if (!note || !words || !at) return null;
  return {
    kind: "unstick",
    at,
    note: Array.from(words.querySelectorAll(`:scope > ${NOTE}`)).indexOf(note),
  };
};

export const boxKey = (event: KeyboardEvent): DayWent | null => {
  if (!(event.target as Element | null)?.closest?.(".diary-box")) return null;
  const keeping = event.key === "Enter" && modKey(event);
  if (event.key !== "Escape" && !keeping) return null;
  event.preventDefault();
  return { kind: keeping ? "dayKeep" : "dayCancel" };
};
