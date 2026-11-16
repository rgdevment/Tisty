import { charsAt, spanned } from "tisty-editor/read";
import type { Beside } from "./pageBeside";
import type { Caret } from "./pageCaret";
import type { Span } from "./pageEdits";
import type { Laying } from "./pageLaying";
import { movedTo, topsOf } from "./pageMoving";
import { endIn } from "./pageSpots";
import type { Writer } from "./pageWriter";

const CHOSEN = "chosen-block";

export type Choosing = ReturnType<typeof choosingOn>;

// Escape chooses the whole block the caret stands in, and Shift with an arrow takes in the blocks beside it.
export const choosingOn = (writer: Writer, caret: Caret, laying: Laying, beside: Beside) => {
  const { page, hands } = writer;
  let chosen: { anchor: number; head: number } | null = null;

  const tops = () => {
    const sheet = page.querySelector(".page");
    return sheet ? topsOf(sheet) : [];
  };
  const ends = () => {
    if (!chosen) return null;
    return [Math.min(chosen.anchor, chosen.head), Math.max(chosen.anchor, chosen.head)] as const;
  };

  const shown = () => {
    for (const one of page.querySelectorAll(`.${CHOSEN}`)) one.classList.remove(CHOSEN);
    const both = ends();
    if (!both) return;
    const all = tops();
    for (let at = both[0]; at <= both[1]; at++) all[at]?.classList.add(CHOSEN);
  };

  const span = (): Span | null => {
    const both = ends();
    const all = tops();
    const first = both && spanned(all[both[0]]?.getAttribute("data-at") ?? null);
    const last = both && spanned(all[both[1]]?.getAttribute("data-at") ?? null);
    if (!first || !last) return null;
    const text = hands.text();
    return { from: charsAt(text, first[0]), to: charsAt(text, last[1]) };
  };

  const letGo = (laid = true) => {
    const head = chosen ? tops()[chosen.head] : null;
    chosen = null;
    shown();
    if (laid && head) caret.laidAt(endIn(head));
  };

  const choose = (event: KeyboardEvent) => {
    const top = caret.blockOfCaret()?.closest(".page > [data-at]");
    const at = top ? tops().indexOf(top) : -1;
    if (at < 0) return;
    event.preventDefault();
    chosen = { anchor: at, head: at };
    page.getSelection()?.removeAllRanges();
    shown();
  };

  const dropped = () => {
    const both = ends();
    const one = span();
    if (!both || !one) return;
    const after = spanned(tops()[both[1] + 1]?.getAttribute("data-at") ?? null);
    const text = hands.text();
    const to = after ? charsAt(text, after[0]) : one.to;
    chosen = null;
    shown();
    laying.put({ from: one.from, to, insert: "" }, one.from);
  };

  const moved = (way: -1 | 1) => {
    const both = ends();
    const one = span();
    if (!both || !one || !chosen) return;
    const all = tops();
    const text = hands.text();
    const past = way < 0 ? all[both[0] - 1] : all[both[1] + 2];
    if (way < 0 && !past) return;
    if (way > 0 && !all[both[1] + 1]) return;
    const at = spanned(past?.getAttribute("data-at") ?? null);
    const target = at ? charsAt(text, at[0]) : text.length;
    beside.replaced(movedTo(text, { ...one, last: both[1] }, target));
    chosen = { anchor: chosen.anchor + way, head: chosen.head + way };
  };

  // While blocks are chosen the keys act on them all; any other key lets them go and writes after them.
  const keyed = (event: KeyboardEvent): boolean => {
    if (!chosen) return false;
    const arrow = event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
    const held = ["Shift", "Control", "Alt", "Meta"].includes(event.key);
    const clipped =
      (event.ctrlKey || event.metaKey) && ["c", "x"].includes(event.key.toLowerCase());
    if (held || clipped) return true;
    const acted =
      event.key === "Escape" ||
      (arrow !== 0 && (event.shiftKey || event.altKey)) ||
      event.key === "Backspace" ||
      event.key === "Delete";
    if (!acted) {
      letGo();
      return false;
    }
    event.preventDefault();
    if (event.key === "Escape") letGo();
    else if (arrow && event.shiftKey) {
      chosen.head = Math.max(0, Math.min(tops().length - 1, chosen.head + arrow));
      shown();
    } else if (arrow) moved(arrow);
    else dropped();
    return true;
  };

  return { choose, keyed, span, shown, letGo, dropped, choosing: () => chosen !== null };
};
