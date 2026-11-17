import { cardBox, glyph, piece } from "./cards";
import type { Say } from "./words";

export type CardStep = { readonly id: string; readonly html: string; readonly done: boolean };

export type CardParts = { readonly closed: number; readonly all: number };

/** What the host read of a cited task, `about` already in the reader's words: its list and its days. */
export type TaskCarded =
  | {
      readonly kind: "open";
      readonly title: string;
      readonly about?: string;
      readonly steps: readonly CardStep[];
      readonly parts?: CardParts;
    }
  | {
      readonly kind: "closed";
      readonly title: string;
      readonly about?: string;
      readonly dropped: boolean;
      readonly steps: readonly CardStep[];
      readonly parts?: CardParts;
    }
  | { readonly kind: "gone" }
  | { readonly kind: "waiting" };

export type TaskCards = {
  readonly task: (id: string) => TaskCarded;
  readonly say: Say;
  readonly acting?: boolean;
};

const UNFOLDED = "unfolded";

const asked = (page: Document, kind: string, id: string, label: string) => {
  const one = page.createElement("button");
  one.type = "button";
  one.dataset.asked = kind;
  one.dataset.task = id;
  one.setAttribute("aria-label", label);
  return one;
};

const tileOf = (page: Document, id: string, seen: TaskCarded, cards: TaskCards) => {
  const tile = piece(page, "card-tile");
  if (seen.kind === "open" && cards.acting) {
    const check = asked(page, "taskDone", id, cards.say("taskCardFinish"));
    check.title = cards.say("taskCardFinish");
    check.classList.add("card-check");
    check.append(glyph(page, "todo"));
    tile.append(check);
    return tile;
  }
  tile.append(glyph(page, seen.kind === "closed" ? "done" : "todo"));
  return tile;
};

const chipsOf = (page: Document, id: string, seen: TaskCarded, say: Say): HTMLElement[] => {
  if (seen.kind === "gone") return [];
  if (seen.kind === "waiting") return [piece(page, "kind", say("taskCardKind"))];
  const chips: HTMLElement[] = [];
  if (seen.parts)
    chips.push(
      piece(page, "chip", say("taskCardParts", `${seen.parts.closed}`, `${seen.parts.all}`)),
    );
  if (seen.steps.length > 0) {
    const done = seen.steps.filter((one) => one.done).length;
    const said = say("partCardSteps", `${done}`, `${seen.steps.length}`);
    const toggle = asked(page, "taskSteps", id, said);
    toggle.className = "chip card-unfold";
    toggle.textContent = said;
    toggle.setAttribute("aria-expanded", "false");
    chips.push(toggle);
  }
  if (seen.kind === "closed")
    chips.push(piece(page, "chip here", say(seen.dropped ? "partCardDropped" : "partCardDone")));
  if (chips.length === 0) chips.push(piece(page, "kind", say("taskCardKind")));
  return chips;
};

const stepsOf = (page: Document, id: string, steps: readonly CardStep[], ticking: boolean) => {
  const all = piece(page, "card-steps");
  for (const step of steps) {
    const row = piece(page, step.done ? "card-step is-done" : "card-step");
    const words = piece(page, "card-step-words");
    words.innerHTML = step.html;
    const box = ticking
      ? asked(page, "taskStep", id, words.textContent ?? "")
      : piece(page, "card-step-box");
    if (ticking) {
      box.classList.add("card-step-box");
      box.dataset.step = step.id;
      box.setAttribute("role", "checkbox");
      box.setAttribute("aria-checked", `${step.done}`);
    }
    row.append(box, words);
    all.append(row);
  }
  return all;
};

const blurbOf = (seen: TaskCarded, say: Say): string => {
  if (seen.kind === "gone") return say("taskCardAway");
  if (seen.kind === "waiting") return "";
  return seen.about ?? "";
};

const carded = (page: Document, id: string, written: string, cards: TaskCards) => {
  const seen = cards.task(id);
  const known = seen.kind === "open" || seen.kind === "closed";
  const box = cardBox(page, `is-task is-${seen.kind}`, seen.kind !== "gone");
  box.dataset.task = id;
  box.dataset.written = written;
  const meta = piece(page, "card-meta");
  meta.append(...chipsOf(page, id, seen, cards.say));
  if (seen.kind !== "gone") meta.append(piece(page, "card-go", "›"));
  box.append(
    tileOf(page, id, seen, cards),
    piece(page, "card-name", known ? seen.title : written),
    piece(page, "card-blurb", blurbOf(seen, cards.say)),
    meta,
  );
  if (known && seen.steps.length > 0)
    box.append(stepsOf(page, id, seen.steps, seen.kind === "open" && Boolean(cards.acting)));
  return box;
};

const shown = (box: HTMLElement, open: boolean) => {
  box.classList.toggle(UNFOLDED, open);
  box.querySelector(".card-unfold")?.setAttribute("aria-expanded", `${open}`);
};

// A task alone in its paragraph is drawn as its card, as a document's is; inside a sentence it stays a word.
export const taskCarded = (page: Document, cards: TaskCards) => {
  for (const link of page.querySelectorAll<HTMLAnchorElement>(".page > p > a.card[data-task]")) {
    const held = link.parentElement;
    if (held?.children.length !== 1) continue;
    if ((held.textContent ?? "").trim() !== (link.textContent ?? "").trim()) continue;
    link.replaceWith(carded(page, link.dataset.task ?? "", link.textContent || "", cards));
  }
};

// The tasks are asked for after the page is drawn, so their cards change in place rather than drawing it again.
export const recarded = (page: Document, cards: TaskCards) => {
  for (const box of page.querySelectorAll<HTMLElement>(".page > p > .is-task[data-task]")) {
    const fresh = carded(page, box.dataset.task ?? "", box.dataset.written ?? "", cards);
    const focused = page.activeElement === box;
    shown(fresh, box.classList.contains(UNFOLDED));
    box.replaceWith(fresh);
    if (focused) fresh.focus();
  }
};

export const unfolded = (page: Document, nth: number) => {
  const box = page.querySelectorAll<HTMLElement>(".page > p > .is-task[data-task]")[nth];
  if (box) shown(box, !box.classList.contains(UNFOLDED));
};

export const taskCardAt = (button: Element): number =>
  Array.from(button.ownerDocument.querySelectorAll(".page > p > .is-task[data-task]")).indexOf(
    button.closest(".is-task") as Element,
  );
