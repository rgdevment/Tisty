import { scrolledAround } from "./pageNodes";
import { placedAt, placeOf } from "./pagePlaces";

const DIAGRAMS_AND_WIDGETS_SETTLE_MS = 2000;
const A_DRAWING_ARRIVES_MS = 2000;
const HANDS = ["wheel", "pointerdown", "keydown"] as const;

export type Change = { readonly from: number; readonly removed: number; readonly by: number };

type Held = { readonly start: number; readonly y: number };

const movedBy = (held: Held, change: Change): Held | null => {
  if (held.start <= change.from) return held;
  if (held.start < change.from + change.removed) return null;
  return { ...held, start: held.start + change.by };
};

export const holdingOn = (page: Document) => {
  let held: Held | null = null;
  let drawingAskedAt: number | null = null;
  let letGo = () => {};

  const scrollingAround = (): HTMLElement | null =>
    page.documentElement.classList.contains("tall")
      ? (scrolledAround(page)[0] ?? null)
      : (page.scrollingElement as HTMLElement | null);
  const frameTop = () => page.defaultView?.frameElement?.getBoundingClientRect().top ?? 0;
  const blockAt = (start: number) => placedAt(page, ".page > ", start);
  const topOf = (block: Element) => block.getBoundingClientRect().top + frameTop();

  const firstSeen = (around: HTMLElement): Held | null => {
    const own = around === page.scrollingElement;
    const sight = own ? 0 : Math.max(0, around.getBoundingClientRect().top - frameTop());
    const blocks = Array.from(page.querySelectorAll(".page > [data-at]:not(.front)"));
    // A hint hidden on a task's page measures nothing, so the blocks are not in order of height to search by halves.
    const block = blocks.find((one) => {
      const box = one.getBoundingClientRect();
      return box.height > 0 && box.bottom > sight;
    });
    const start = placeOf(block)?.[0];
    return block && start !== undefined ? { start, y: topOf(block) } : null;
  };

  const before = (change: Change) => {
    const asked = performance.now();
    const onItsWay = drawingAskedAt !== null && asked - drawingAskedAt < A_DRAWING_ARRIVES_MS;
    const around = scrollingAround();
    const seen = onItsWay ? held : around && firstSeen(around);
    held = seen && movedBy(seen, change);
    if (!onItsWay) drawingAskedAt = asked;
  };

  const pinned = (around: HTMLElement, { start, y }: Held) => {
    const block = blockAt(start);
    if (block) around.scrollTop += topOf(block) - y;
  };

  const keptWhileSettling = (around: HTMLElement, one: Held, caretInSight: () => void) => {
    letGo();
    const sheet = page.querySelector(".page");
    const Seeing = page.defaultView?.ResizeObserver;
    if (!sheet || !Seeing) return;
    // Blocks drawn anew are first laid at a guessed height, so the caret is brought into sight again once they take their own.
    const seeing = new Seeing(() => {
      pinned(around, one);
      caretInSight();
    });
    seeing.observe(sheet);
    const pageAndItsWindow = [page, page.defaultView?.frameElement?.ownerDocument];
    const off = () => {
      seeing.disconnect();
      clearTimeout(settled);
      for (const at of pageAndItsWindow)
        for (const hand of HANDS) at?.removeEventListener(hand, off, true);
      letGo = () => {};
    };
    const settled = setTimeout(off, DIAGRAMS_AND_WIDGETS_SETTLE_MS);
    for (const at of pageAndItsWindow)
      for (const hand of HANDS) at?.addEventListener(hand, off, { capture: true, passive: true });
    letGo = off;
  };

  const landed = (layCaret: () => void, caretInSight: () => void) => {
    const one = held;
    held = null;
    drawingAskedAt = null;
    const around = scrollingAround();
    if (!one || !around || !blockAt(one.start)) return layCaret();
    pinned(around, one);
    layCaret();
    keptWhileSettling(around, one, caretInSight);
  };

  return { before, landed, stop: () => letGo() };
};
