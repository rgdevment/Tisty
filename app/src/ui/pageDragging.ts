import type { Caret } from "./pageCaret";
import type { Edit, Span } from "./pageEdits";
import type { Laying } from "./pageLaying";
import { balanced } from "./pageMarks";
import { FIELDS } from "./pageNodes";
import type { Writer } from "./pageWriter";

const STILL_PX = 4;

type Pointing = Document & {
  caretPositionFromPoint?: (x: number, y: number) => { offsetNode: Node; offset: number } | null;
};

const pointAt = (page: Document, x: number, y: number): Range | null => {
  const range = page.caretRangeFromPoint?.(x, y);
  if (range) return range;
  const spot = (page as Pointing).caretPositionFromPoint?.(x, y);
  if (!spot) return null;
  const made = page.createRange();
  made.setStart(spot.offsetNode, spot.offset);
  return made;
};

export const movedWords = (text: string, words: Span, to: number): [Edit, number] | null => {
  if (to >= words.from && to <= words.to) return null;
  const said = text.slice(words.from, words.to);
  if (to < words.from)
    return [
      { from: to, to: words.to, insert: said + text.slice(to, words.from) },
      to + said.length,
    ];
  return [{ from: words.from, to, insert: text.slice(words.to, to) + said }, to];
};

// The window takes the system's drags for its files, so words are carried by the pointer, as a block by its grip.
export const draggingOn = (writer: Writer, caret: Caret, laying: Laying) => {
  const { page, hands } = writer;
  const mark = page.createElement("div");
  mark.className = "page-drop-caret";
  mark.hidden = true;
  page.body.append(mark);
  let held: { x: number; y: number; words: Span; said: string; moved: boolean } | null = null;

  const shown = (point: Range | null) => {
    const box = point?.getClientRects()[0] ?? point?.getBoundingClientRect();
    const view = page.defaultView;
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    if (!box || !view) return;
    mark.hidden = false;
    mark.style.top = `${(box.top + view.scrollY) / zoom}px`;
    mark.style.left = `${(box.left + view.scrollX) / zoom}px`;
    mark.style.height = `${Math.max(box.height, 16) / zoom}px`;
  };

  const dragged = (event: MouseEvent) => {
    if (!held) return;
    const far = Math.hypot(event.clientX - held.x, event.clientY - held.y) > STILL_PX;
    if (!held.moved && !far) return;
    held.moved = true;
    shown(pointAt(page, event.clientX, event.clientY));
  };

  const dropped = (event: MouseEvent) => {
    page.removeEventListener("mousemove", dragged);
    page.removeEventListener("mouseup", dropped);
    mark.hidden = true;
    const was = held;
    held = null;
    const point = pointAt(page, event.clientX, event.clientY);
    if (!was || !point) return;
    page.querySelector<HTMLElement>(".page")?.focus({ preventScroll: true });
    if (!was.moved) return page.getSelection()?.collapse(point.startContainer, point.startOffset);
    const to = caret.spanOfRange(point)?.from;
    const text = hands.text();
    if (text.slice(was.words.from, was.words.to) !== was.said) return;
    const moved = to === undefined ? null : movedWords(text, was.words, to);
    if (moved) laying.put(...moved);
  };

  const pressed = (event: MouseEvent) => {
    if (event.button !== 0 || event.shiftKey || event.detail > 1) return;
    const target = event.target as Element | null;
    if (!target?.closest?.(".page") || target.closest(FIELDS)) return;
    const selection = page.getSelection();
    const range = selection?.rangeCount ? selection.getRangeAt(0) : null;
    const point = pointAt(page, event.clientX, event.clientY);
    if (!range || range.collapsed || !point) return;
    if (range.comparePoint(point.startContainer, point.startOffset) !== 0) return;
    const chosen = caret.chosenNow();
    if (!chosen) return;
    event.preventDefault();
    const text = hands.text();
    const words = balanced(text, chosen);
    held = {
      x: event.clientX,
      y: event.clientY,
      words,
      said: text.slice(words.from, words.to),
      moved: false,
    };
    page.addEventListener("mousemove", dragged);
    page.addEventListener("mouseup", dropped);
  };

  page.addEventListener("mousedown", pressed, true);
  return {
    stop: () => {
      page.removeEventListener("mousedown", pressed, true);
      page.removeEventListener("mousemove", dragged);
      page.removeEventListener("mouseup", dropped);
      mark.remove();
    },
  };
};
