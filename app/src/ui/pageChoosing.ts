import { charsAt } from "tisty-editor/read";
import { letterOf, modKey, moveKey } from "./macKeys";
import type { Beside } from "./pageBeside";
import type { Caret } from "./pageCaret";
import type { Span } from "./pageEdits";
import type { Laying } from "./pageLaying";
import { movedTo, swappedItem, topsOf } from "./pageMoving";
import { sectioned, WRITABLE } from "./pageNodes";
import { placeOf } from "./pagePlaces";
import { endIn } from "./pageSpots";
import type { Writer } from "./pageWriter";

const CHOSEN = "chosen-block";

export type Choosing = ReturnType<typeof choosingOn>;

// Escape chooses the list item or else the block the caret stands in, and Escape again or Shift with an arrow takes in more.
export const choosingOn = (writer: Writer, caret: Caret, laying: Laying, beside: Beside) => {
  const { page, hands } = writer;
  let chosen: { anchor: number; head: number } | null = null;
  let item: Element | null = null;

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
    if (item) return item.classList.add(CHOSEN);
    const both = ends();
    if (!both) return;
    const all = tops();
    for (let at = both[0]; at <= both[1]; at++) all[at]?.classList.add(CHOSEN);
  };

  // An item's place runs over the blank lines after it, which belong to the list, and starts past its quote marks.
  const itemSpan = (one: Element): Span | null => {
    const at = placeOf(one);
    if (!at) return null;
    const text = hands.text();
    const from = text.lastIndexOf("\n", charsAt(text, at[0]) - 1) + 1;
    const to = from + text.slice(from, charsAt(text, at[1])).replace(/[\r\n]+$/, "").length;
    return { from, to };
  };

  const span = (): Span | null => {
    if (item) return itemSpan(item);
    const both = ends();
    const all = tops();
    const first = both && placeOf(all[both[0]]);
    const last = both && placeOf(all[both[1]]);
    if (!first || !last) return null;
    const text = hands.text();
    return { from: charsAt(text, first[0]), to: charsAt(text, last[1]) };
  };

  // The caret goes after the chosen words it can stand in, never into a table or a drawing.
  const letGo = (laid = true) => {
    if (item) {
      const was = item;
      item = null;
      shown();
      if (laid) caret.laidAt(endIn(was));
      return;
    }
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
  const fixed = (one: Element | undefined) =>
    Boolean(one?.matches("h1, h2, h3, h4, h5, h6")) && sectioned(page);

  const choose = (event: KeyboardEvent) => {
    const block = caret.blockOfCaret();
    const top = block?.closest(".page > [data-at]");
    const all = tops();
    const at = top ? all.indexOf(top) : -1;
    if (!top || at < 0 || fixed(all[at])) return;
    event.preventDefault();
    const one = block?.closest("li[data-at]");
    if (one && top.querySelectorAll("li[data-at]").length > 1) item = one;
    else chosen = { anchor: at, head: at };
    page.getSelection()?.removeAllRanges();
    shown();
  };

  const widened = () => {
    const top = item?.closest(".page > [data-at]");
    const at = top ? tops().indexOf(top) : -1;
    item = null;
    if (at >= 0) chosen = { anchor: at, head: at };
    shown();
  };

  const dropped = () => {
    if (item) {
      const one = itemSpan(item);
      item = null;
      shown();
      if (!one) return;
      const text = hands.text();
      let to = one.to;
      if (text[to] === "\r") to++;
      if (text[to] === "\n") to++;
      return laying.put({ from: one.from, to, insert: "" }, one.from);
    }
    const both = ends();
    const one = span();
    if (!both || !one) return;
    const after = placeOf(tops()[both[1] + 1]);
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
    const at = placeOf(past);
    const target = at ? charsAt(text, at[0]) : text.length;
    const next = movedTo(text, { ...one, last: both[1] }, target);
    if (!next) return;
    beside.replaced(next);
    chosen = { anchor: chosen.anchor + way, head: chosen.head + way };
  };

  // While blocks are chosen the keys act on them all; any other key lets them go and writes after them.
  const keyed = (event: KeyboardEvent): boolean => {
    if (!chosen && !item) return false;
    const arrow = event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
    const held = ["Shift", "Control", "Alt", "Meta"].includes(event.key);
    const clipped = modKey(event) && ["c", "x"].includes(letterOf(event));
    if (held || clipped) return true;
    const acted =
      event.key === "Escape" ||
      (arrow !== 0 && (event.shiftKey || moveKey(event))) ||
      event.key === "Backspace" ||
      event.key === "Delete";
    if (!acted) {
      letGo();
      return false;
    }
    event.preventDefault();
    if (event.key === "Escape") {
      if (item) widened();
      else letGo();
      return true;
    }
    if (item && arrow && !event.shiftKey) {
      const swapped = swappedItem(hands.text(), item, arrow);
      if (!swapped) return true;
      letGo(false);
      beside.replaced(swapped);
      return true;
    }
    if (item && arrow) widened();
    if (!chosen && arrow) return true;
    if (chosen && arrow && event.shiftKey) {
      const all = tops();
      const next = Math.max(0, Math.min(all.length - 1, chosen.head + arrow));
      if (!fixed(all[next])) chosen.head = next;
      shown();
    } else if (arrow) moved(arrow);
    else dropped();
    return true;
  };

  return {
    choose,
    keyed,
    span,
    shown,
    letGo,
    dropped,
    choosing: () => chosen !== null || item !== null,
  };
};
