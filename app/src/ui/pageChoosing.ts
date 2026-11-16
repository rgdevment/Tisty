import { charsAt, spanned } from "tisty-editor/read";
import type { Beside } from "./pageBeside";
import type { Caret } from "./pageCaret";
import type { Span } from "./pageEdits";
import type { Laying } from "./pageLaying";
import { movedTo, topsOf } from "./pageMoving";
import { WRITABLE } from "./pageNodes";
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

  // The caret goes after the chosen words it can stand in, never into a table or a drawing.
  const letGo = (laid = true) => {
    const both = ends();
    const all = tops();
    const head = chosen ? all[chosen.head] : null;
    const toward =
      both && head
        ? chosen?.head === both[0]
          ? all.slice(both[0], both[1] + 1)
          : all.slice(both[0], both[1] + 1).reverse()
        : [];
    const writable = toward.find((one) => one.matches(WRITABLE));
    chosen = null;
    shown();
    if (laid && writable) caret.laidAt(endIn(writable));
  };
  const fixed = (one: Element | undefined) => {
    const look = page.documentElement.classList;
    return (
      Boolean(one?.matches("h1, h2, h3, h4, h5, h6")) &&
      look.contains("untitled") &&
      !look.contains("shaping")
    );
  };

  const choose = (event: KeyboardEvent) => {
    const top = caret.blockOfCaret()?.closest(".page > [data-at]");
    const all = tops();
    const at = top ? all.indexOf(top) : -1;
    if (at < 0 || fixed(all[at])) return;
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
    let from = one.from;
    if (!after) while (text[from - 1] === "\n" && text[from - 2] === "\n") from--;
    const to = after ? charsAt(text, after[0]) : text.length;
    chosen = null;
    shown();
    laying.put({ from, to, insert: "" }, from);
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
    const next = movedTo(text, { ...one, last: both[1] }, target);
    if (!next) return;
    beside.replaced(next);
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
      const all = tops();
      const next = Math.max(0, Math.min(all.length - 1, chosen.head + arrow));
      if (!fixed(all[next])) chosen.head = next;
      shown();
    } else if (arrow) moved(arrow);
    else dropped();
    return true;
  };

  return { choose, keyed, span, shown, letGo, dropped, choosing: () => chosen !== null };
};
