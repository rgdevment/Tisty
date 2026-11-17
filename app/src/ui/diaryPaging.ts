import type { DayAt } from "../diary";
import { t } from "../locales";

export type DayWent =
  | { kind: "day"; at: DayAt }
  | { kind: "dayKeep" }
  | { kind: "dayCancel" }
  | { kind: "unstick"; at: DayAt; note: number };

const NOTE = 'blockquote.callout[data-callout="idea"]';

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
  const keeping = event.key === "Enter" && (event.ctrlKey || event.metaKey);
  if (event.key !== "Escape" && !keeping) return null;
  event.preventDefault();
  return { kind: keeping ? "dayKeep" : "dayCancel" };
};

export const unstickable = (page: Document) => {
  for (const note of page.querySelectorAll(`.diary-words > ${NOTE}`)) {
    if (note.querySelector(":scope > .note-off")) continue;
    const off = page.createElement("button");
    off.type = "button";
    off.className = "note-off";
    off.dataset.asked = "unstick";
    off.setAttribute("aria-label", t("diaryUnstick"));
    off.title = t("diaryUnstick");
    note.append(off);
  }
};
