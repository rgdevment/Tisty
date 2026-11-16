import { spanned } from "tisty-editor/read";
import { carrying } from "./carrying";

const NOTE = '.page > blockquote[data-callout="idea"][data-at]';
const FLOATED = 'blockquote[data-callout="idea"][data-side]';
const LANDS = `.page > [data-at]:not(.front, ${FLOATED})`;
const ABOVE = "idea-landing-above";
const BELOW = "idea-landing-below";

type Landing = { readonly block: HTMLElement; readonly above: boolean };
type Moved = (note: [number, number], to: number) => void;

const landsOf = (page: Document, note: HTMLElement) =>
  Array.from(page.querySelectorAll<HTMLElement>(LANDS)).filter((one) => one !== note);

const landingAt = (page: Document, note: HTMLElement, y: number): Landing | null => {
  const blocks = landsOf(page, note);
  for (const block of blocks) {
    const edge = block.getBoundingClientRect();
    if (y < edge.bottom) return { block, above: y < edge.top + edge.height / 2 };
  }
  const last = blocks[blocks.length - 1];
  return last ? { block: last, above: false } : null;
};

const besideOf = (page: Document, note: HTMLElement, up: boolean): Landing | null => {
  const blocks = landsOf(page, note);
  const after = (one: HTMLElement) =>
    Boolean(note.compareDocumentPosition(one) & Node.DOCUMENT_POSITION_FOLLOWING);
  const block = up ? blocks.filter((one) => !after(one)).pop() : blocks.find((one) => after(one));
  return block ? { block, above: up } : null;
};

const shown = (page: Document, landing: Landing | null) => {
  for (const one of page.querySelectorAll(`.${ABOVE}, .${BELOW}`))
    one.classList.remove(ABOVE, BELOW);
  landing?.block.classList.add(landing.above ? ABOVE : BELOW);
};

const landed = (at: [number, number], landing: Landing | null, moved: Moved) => {
  const there = landing && spanned(landing.block.dataset.at ?? null);
  if (landing && there) moved(at, landing.above ? there[0] : there[1]);
};

const WORDS = ".check, [data-src], [data-ghost], a, button, input";

export const stickable = (page: Document) => {
  for (const note of page.querySelectorAll<HTMLElement>(NOTE)) note.tabIndex = 0;
};

export const sticking = (page: Document, moving: () => Moved | undefined) => {
  page.addEventListener("pointerdown", (event) => {
    const target = event.target as Element | null;
    const note = target?.closest?.<HTMLElement>(NOTE);
    const at = spanned(note?.dataset.at ?? null);
    const moved = moving();
    if (!note || !at || !moved || event.button !== 0 || note.closest(".editing")) return;
    // A note is carried by its paper, tape or pin; its words and boxes are written and ticked.
    if (target?.closest(WORDS)) return;
    let landing: Landing | null = null;
    carrying(page, note, event, 6, {
      over: (next) => {
        note.classList.add("idea-moving");
        landing = landingAt(page, note, next.clientY);
        shown(page, landing);
      },
      ended: () => {
        note.classList.remove("idea-moving");
        shown(page, null);
      },
      dropped: () => landed(at, landing, moved),
    });
  });
  page.addEventListener("keydown", (event) => {
    if (!event.altKey || (event.key !== "ArrowUp" && event.key !== "ArrowDown")) return;
    const note = (event.target as Element | null)?.closest?.<HTMLElement>(NOTE);
    const at = spanned(note?.dataset.at ?? null);
    const moved = moving();
    if (!note || !at || !moved) return;
    event.preventDefault();
    landed(at, besideOf(page, note, event.key === "ArrowUp"), moved);
  });
};

export const movedBlock = (text: string, [from, end]: [number, number], to: number): string => {
  let cut = end;
  while (cut < text.length && (text[cut] === "\n" || text[cut] === "\r")) cut++;
  const home = to <= from ? /^[\r\n]*$/.test(text.slice(to, from)) : to <= cut;
  if (home) return text;
  const ending = text.includes("\r\n") ? "\r\n" : "\n";
  const block = text.slice(from, end).replace(/[\r\n]+$/, "");
  const rest = `${text.slice(0, from)}${text.slice(cut)}`;
  const at = to > end ? to - (cut - from) : to;
  const before = rest.slice(0, at).replace(/[\r\n]+$/, "");
  const after = rest.slice(at).replace(/^[\r\n]+/, "");
  const gap = `${ending}${ending}`;
  return `${before}${before ? gap : ""}${block}${after ? `${gap}${after}` : ending}`;
};
