import { spanned } from "tisty-editor/read";

const NOTE = '.page > blockquote[data-callout="idea"][data-at]';
const LANDS = ".page > [data-at]:not(.front)";
const ABOVE = "idea-landing-above";
const BELOW = "idea-landing-below";

type Landing = { readonly block: HTMLElement; readonly above: boolean };

const landingAt = (page: Document, note: HTMLElement, y: number): Landing | null => {
  const blocks = Array.from(page.querySelectorAll<HTMLElement>(LANDS)).filter(
    (one) => one !== note,
  );
  for (const block of blocks) {
    const edge = block.getBoundingClientRect();
    if (y < edge.bottom) return { block, above: y < edge.top + edge.height / 2 };
  }
  const last = blocks[blocks.length - 1];
  return last ? { block: last, above: false } : null;
};

const shown = (page: Document, landing: Landing | null) => {
  for (const one of page.querySelectorAll(`.${ABOVE}, .${BELOW}`))
    one.classList.remove(ABOVE, BELOW);
  landing?.block.classList.add(landing.above ? ABOVE : BELOW);
};

type Moved = (note: [number, number], to: number) => void;

export const sticking = (page: Document, moving: () => Moved | undefined) => {
  page.addEventListener("pointerdown", (event) => {
    const note = (event.target as Element | null)?.closest?.<HTMLElement>(NOTE);
    const at = spanned(note?.dataset.at ?? null);
    const moved = moving();
    if (!note || !at || !moved || event.button !== 0 || note.closest(".editing")) return;
    const startY = event.clientY;
    let far = false;
    let landing: Landing | null = null;
    const over = (next: PointerEvent) => {
      far ||= Math.abs(next.clientY - startY) > 6;
      if (!far) return;
      next.preventDefault();
      note.classList.add("idea-moving");
      landing = landingAt(page, note, next.clientY);
      shown(page, landing);
    };
    const ended = () => {
      page.removeEventListener("pointermove", over);
      page.removeEventListener("pointerup", up);
      page.removeEventListener("pointercancel", ended);
      note.classList.remove("idea-moving");
      shown(page, null);
    };
    const up = () => {
      ended();
      if (!far) return;
      // The click that ends a carry is not a press on the note, which would open it to write.
      const swallowed = (one: Event) => one.stopPropagation();
      page.addEventListener("click", swallowed, { capture: true, once: true });
      setTimeout(() => page.removeEventListener("click", swallowed, { capture: true }), 0);
      const there = landing && spanned(landing.block.dataset.at ?? null);
      if (landing && there) moved(at, landing.above ? there[0] : there[1]);
    };
    page.addEventListener("pointermove", over);
    page.addEventListener("pointerup", up);
    page.addEventListener("pointercancel", ended);
  });
};

export const movedBlock = (text: string, [from, end]: [number, number], to: number): string => {
  if (to >= from && to <= end) return text;
  const ending = text.includes("\r\n") ? "\r\n" : "\n";
  const block = text.slice(from, end).replace(/[\r\n]+$/, "");
  let cut = end;
  while (cut < text.length && (text[cut] === "\n" || text[cut] === "\r")) cut++;
  const rest = `${text.slice(0, from)}${text.slice(cut)}`;
  const at = to > end ? to - (cut - from) : to;
  const before = rest.slice(0, at).replace(/[\r\n]+$/, "");
  const after = rest.slice(at).replace(/^[\r\n]+/, "");
  const gap = `${ending}${ending}`;
  return `${before}${before ? gap : ""}${block}${after ? `${gap}${after}` : ending}`;
};
