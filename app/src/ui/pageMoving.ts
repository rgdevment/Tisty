import { charsAt, spanned } from "tisty-editor/read";
import { movedBlock } from "./sticking";

export type Unit = { readonly from: number; readonly to: number; readonly last: number };
/** `caret` is where the moved words now start; `was` and `size` are where they stood before. */
export type Moved = {
  readonly text: string;
  readonly caret: number;
  readonly was: number;
  readonly size: number;
};
export type Act = "up" | "down" | "double" | "drop";

const spanOf = (one: Element) => spanned(one.getAttribute("data-at"));
const level = (one: Element) => Number(/^H([1-6])$/.exec(one.tagName)?.[1] ?? 0);
const LIST_MARK = /^(\s*(?:[-*+]|\d+[.)])\s+)/;

export const topsOf = (sheet: Element): Element[] =>
  Array.from(sheet.querySelectorAll(":scope > [data-at]")).filter((one) => !one.matches(".front"));

// A title carries the section under it, up to the next title as high as itself.
export const unitOf = (body: string, tops: readonly Element[], at: number): Unit | null => {
  const top = tops[at];
  if (!top) return null;
  const high = level(top);
  let last = at;
  if (high > 0)
    while (last + 1 < tops.length && !(level(tops[last + 1]) > 0 && level(tops[last + 1]) <= high))
      last++;
  const first = spanOf(top);
  const end = spanOf(tops[last]);
  return first && end ? { from: charsAt(body, first[0]), to: charsAt(body, end[1]), last } : null;
};

const nearest = (text: string, said: string, near: number) => {
  let best = -1;
  for (let at = text.indexOf(said); at >= 0; at = text.indexOf(said, at + 1))
    if (best < 0 || Math.abs(at - near) < Math.abs(best - near)) best = at;
  return Math.max(0, best);
};

export const movedTo = (text: string, unit: Unit, target: number): Moved | null => {
  const next = movedBlock(text, [unit.from, unit.to], target);
  if (next === text) return null;
  const block = text.slice(unit.from, unit.to).replace(/[\r\n]+$/, "");
  const guess = target > unit.to ? target - (unit.to - unit.from) : target;
  return { text: next, caret: nearest(next, block, guess), was: unit.from, size: block.length };
};

// An item trades places with the one beside it, each place keeping its own mark and number.
export const swappedItem = (body: string, item: Element, way: -1 | 1): Moved | null => {
  const beside = way < 0 ? item.previousElementSibling : item.nextElementSibling;
  if (!beside?.matches("li[data-at]")) return null;
  const [upper, lower] = way < 0 ? [beside, item] : [item, beside];
  const up = spanOf(upper);
  const low = spanOf(lower);
  if (!up || !low) return null;
  const [a0, a1, b0, b1] = [up[0], up[1], low[0], low[1]].map((one) => charsAt(body, one));
  const said = body.slice(a0, a1);
  const other = body.slice(b0, b1);
  const markA = LIST_MARK.exec(said)?.[1] ?? "";
  const markB = LIST_MARK.exec(other)?.[1] ?? "";
  const first = `${markA}${other.slice(markB.length)}`;
  const second = `${markB}${said.slice(markA.length)}`;
  const text = `${body.slice(0, a0)}${first}${body.slice(a1, b0)}${second}${body.slice(b1)}`;
  const caret = way < 0 ? a0 + markA.length : a0 + first.length + (b0 - a1) + markB.length;
  const moving = way < 0 ? other.slice(markB.length) : said.slice(markA.length);
  const was = way < 0 ? b0 + markB.length : a0 + markA.length;
  return { text, caret, was, size: moving.length };
};

export const acted = (
  body: string,
  tops: readonly Element[],
  at: number,
  act: Act,
): Moved | null => {
  const unit = unitOf(body, tops, at);
  if (!unit) return null;
  if (act === "up") {
    const above = at > 0 ? spanOf(tops[at - 1]) : null;
    return above ? movedTo(body, unit, charsAt(body, above[0])) : null;
  }
  if (act === "down") {
    const below = unitOf(body, tops, unit.last + 1);
    return below ? movedTo(body, unit, below.to) : null;
  }
  const said = body.slice(unit.from, unit.to);
  if (act === "double")
    return {
      text: `${body.slice(0, unit.to)}\n\n${said}${body.slice(unit.to)}`,
      caret: unit.to + 2,
      was: unit.from,
      size: said.length,
    };
  let cut = unit.to;
  while (cut < body.length && /[\r\n]/.test(body[cut] ?? "")) cut++;
  return {
    text: `${body.slice(0, unit.from)}${body.slice(cut)}`,
    caret: unit.from,
    was: unit.from,
    size: 0,
  };
};

type Gripping = {
  readonly moved: (top: Element, target: Element | null) => void;
  readonly asked: (top: Element, at: { x: number; y: number }) => void;
};

const STILL_PX = 4;

// A block is moved by the grip that stands beside it on the sheet's left edge; a press on the grip asks what to do with it.
export const gripped = (page: Document, hands: Gripping): (() => void) => {
  const grip = page.createElement("button");
  grip.type = "button";
  grip.className = "page-grip";
  grip.textContent = "⠿";
  grip.hidden = true;
  const line = page.createElement("div");
  line.className = "page-drop";
  line.hidden = true;
  page.body.append(grip, line);
  const view = page.defaultView;
  const zoom = () => Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
  let over: Element | null = null;
  let held: { y: number; moved: boolean; target: Element | null } | null = null;

  const placeOn = (one: HTMLElement, top: number, inset: number) => {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    one.style.top = `${(top + (view?.scrollY ?? 0)) / zoom()}px`;
    one.style.left = `${(sheet.getBoundingClientRect().left + (view?.scrollX ?? 0)) / zoom() + inset}px`;
  };

  const hovered = (event: MouseEvent) => {
    if (held) return;
    const target = event.target as Element | null;
    if (target === grip) return;
    const top = target?.closest?.(".page > [data-at]");
    if (!top || top.matches(".front")) return;
    over = top;
    grip.hidden = false;
    placeOn(grip, top.getBoundingClientRect().top + 2, 1);
  };

  const targetAt = (y: number): Element | null => {
    const sheet = page.querySelector(".page");
    if (!sheet) return null;
    return (
      topsOf(sheet).find((one) => {
        const box = one.getBoundingClientRect();
        return y < box.top + box.height / 2;
      }) ?? null
    );
  };

  const dragged = (event: MouseEvent) => {
    if (!held) return;
    if (Math.abs(event.clientY - held.y) > STILL_PX) held.moved = true;
    if (!held.moved) return;
    held.target = targetAt(event.clientY);
    const sheet = page.querySelector(".page");
    const all = sheet ? topsOf(sheet) : [];
    const last = all[all.length - 1];
    const edge = held.target?.getBoundingClientRect().top ?? last?.getBoundingClientRect().bottom;
    if (edge === undefined) return;
    line.hidden = false;
    placeOn(line, edge - 1, 8);
    if (sheet) line.style.width = `${sheet.getBoundingClientRect().width / zoom() - 16}px`;
  };

  const letGo = () => {
    const was = held;
    held = null;
    line.hidden = true;
    page.removeEventListener("mousemove", dragged);
    page.removeEventListener("mouseup", letGo);
    if (!was || !over) return;
    if (was.moved) return hands.moved(over, was.target);
    const box = grip.getBoundingClientRect();
    const frame = view?.frameElement?.getBoundingClientRect();
    hands.asked(over, { x: (frame?.left ?? 0) + box.left, y: (frame?.top ?? 0) + box.bottom + 4 });
  };

  grip.addEventListener("mousedown", (event) => {
    if (event.button !== 0) return;
    event.preventDefault();
    held = { y: event.clientY, moved: false, target: null };
    page.addEventListener("mousemove", dragged);
    page.addEventListener("mouseup", letGo);
  });
  page.addEventListener("mousemove", hovered);
  return () => {
    page.removeEventListener("mousemove", hovered);
    page.removeEventListener("mousemove", dragged);
    page.removeEventListener("mouseup", letGo);
    grip.remove();
    line.remove();
  };
};
