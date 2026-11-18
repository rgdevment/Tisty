export type CardAsked = {
  readonly kind: "doc" | "task";
  readonly id: string;
  readonly words: string;
};

export type DrawnCards = ReadonlyMap<string, string>;

const UNFOLDED = "unfolded";
const LINKS = ".page a.card[data-doc], .page a.card[data-task]";
const LONE = ".page > p > a.card[data-doc], .page > p > a.card[data-task]";
const PLACED = ".page > p[data-card]";
const TASKS = ".page > p > .is-task[data-task]";
const WRITTEN = /<a class="card" data-(doc|task)="([^"]*)">([^<]*)<\/a>/g;

export const cardKey = (one: CardAsked): string => `${one.kind}\n${one.id}\n${one.words}`;

const keyed = (key: string): CardAsked => {
  const [kind, id, ...words] = key.split("\n");
  return { kind: kind === "task" ? "task" : "doc", id: id ?? "", words: words.join("\n") };
};

const unescaped = (said: string) =>
  said
    .replace(/&quot;/g, '"')
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&");

// The core writes every card as this one link, so a page names its cards before it is laid out.
export const cardsIn = (html: string): CardAsked[] =>
  Array.from(html.matchAll(WRITTEN), ([, kind, id, words]) => ({
    kind: kind === "task" ? "task" : "doc",
    id: unescaped(id),
    words: unescaped(words),
  }));

const linked = (link: HTMLElement): CardAsked =>
  link.dataset.doc === undefined
    ? { kind: "task", id: link.dataset.task ?? "", words: link.textContent ?? "" }
    : { kind: "doc", id: link.dataset.doc, words: link.textContent ?? "" };

export const cardsOn = (page: Document): CardAsked[] =>
  Array.from(page.querySelectorAll<HTMLElement>(`${LINKS}, ${PLACED}`), (one) =>
    one.dataset.card === undefined ? linked(one) : keyed(one.dataset.card),
  );

// A card alone in its paragraph is drawn whole; one inside a sentence stays a word of it.
const lone = (link: Element) => {
  const held = link.parentElement;
  return (
    held?.children.length === 1 &&
    (held.textContent ?? "").trim() === (link.textContent ?? "").trim()
  );
};

const shown = (box: Element, open: boolean) => {
  box.classList.toggle(UNFOLDED, open);
  box.querySelector(".card-unfold")?.setAttribute("aria-expanded", `${open}`);
};

const pressable = (box: HTMLElement) => {
  if (box.getAttribute("role") !== "link") return;
  box.addEventListener("keydown", (event) => {
    if (event.target !== box || (event.key !== "Enter" && event.key !== " ")) return;
    event.preventDefault();
    box.click();
  });
};

const refocused = (box: HTMLElement, held: HTMLElement) => {
  if (held.classList.contains("doc-card")) return box.focus();
  const same = Array.from(box.querySelectorAll<HTMLElement>("button[data-asked]")).find(
    (one) => one.dataset.asked === held.dataset.asked && one.dataset.step === held.dataset.step,
  );
  same?.focus();
};

const laid = (held: HTMLElement, key: string, html: string, days?: (box: Element) => void) => {
  const open = held.firstElementChild?.classList.contains(UNFOLDED) ?? false;
  const focus = held.ownerDocument.activeElement;
  // A page drawn in a frame has its own HTMLElement, so the frame's elements fail an instanceof check.
  const inside = focus && held.contains(focus) ? (focus as HTMLElement) : null;
  held.dataset.card = key;
  held.innerHTML = html;
  const box = held.firstElementChild as HTMLElement | null;
  if (!box) return;
  pressable(box);
  if (open) shown(box, true);
  days?.(box);
  if (inside) refocused(box, inside);
};

export const cardsPlaced = (
  page: Document,
  drawn: DrawnCards,
  was?: DrawnCards,
  days?: (box: Element) => void,
): CardAsked[] => {
  if (was)
    for (const held of page.querySelectorAll<HTMLElement>(PLACED)) {
      const key = held.dataset.card ?? "";
      const html = drawn.get(key);
      if (html !== undefined && html !== was.get(key)) laid(held, key, html, days);
    }
  const missing: CardAsked[] = [];
  for (const link of page.querySelectorAll<HTMLElement>(LONE)) {
    if (!lone(link)) continue;
    const asked = linked(link);
    const html = drawn.get(cardKey(asked));
    if (html === undefined) missing.push(asked);
    else laid(link.parentElement as HTMLElement, cardKey(asked), html, days);
  }
  return missing;
};

export const unfolded = (page: Document, nth: number) => {
  const box = page.querySelectorAll(TASKS)[nth];
  if (box) shown(box, !box.classList.contains(UNFOLDED));
};

export const taskCardAt = (button: Element): number =>
  Array.from(button.ownerDocument.querySelectorAll(TASKS)).indexOf(
    button.closest(".is-task") as Element,
  );
