import { byteFrom, type Reckoned, unitFrom } from "../placing";
import { placeOf } from "./pagePlaces";

const PLACED = "[data-src]";
const BLOCK = "[data-at]";

const spanOf = (one: Element) => placeOf(one, "data-src");
const solid = (one: Element) => one.getAttribute("contenteditable") === "false";

const placedFrom = (tops: HTMLCollection, at: number, until: number) => {
  for (let next = at; next < until; next++) {
    const place = placeOf(tops[next]);
    if (place) return place;
  }
  return null;
};

// The blocks at the top of the sheet lie in the order of their places, so the first that reaches a byte is found by halving.
const firstReaching = (sheet: Element, byte: number, past: boolean) => {
  const tops = sheet.children;
  let low = 0;
  let high = tops.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    const place = placedFrom(tops, middle, high);
    if (!place || (past ? place[1] > byte : place[1] >= byte)) high = middle;
    else low = middle + 1;
  }
  return low;
};

export const topsAt = (sheet: Element, byte: number): Element[] => {
  const found: Element[] = [];
  const tops = sheet.children;
  for (let at = firstReaching(sheet, byte, false); at < tops.length; at++) {
    const place = placeOf(tops[at]);
    if (!place) continue;
    if (place[0] > byte) break;
    found.push(tops[at]);
  }
  return found;
};

const pointIn = (page: Document, node: Node, offset: number) => {
  const point = page.createRange();
  point.setStart(node, offset);
  return point;
};

// A point between pieces of words takes the nearest one in its own block, or the block's end when it has none.
// A point on the sheet outside every block, as its margin or a page with nothing yet, writes after the block before it.
const between = (
  body: string,
  sheet: Element,
  node: Node,
  offset: number,
  known: Reckoned | null,
): number => {
  const page = sheet.ownerDocument;
  const tops = sheet.children;
  const reached = (one: Element) => {
    const start = page.createRange();
    start.setStartBefore(one);
    return start.comparePoint(node, offset) <= 0;
  };
  let low = 0;
  let high = tops.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    if (reached(tops[middle])) high = middle;
    else low = middle + 1;
  }
  let end = 0;
  for (let at = low - 1; at >= 0 && end === 0; at--) end = placeOf(tops[at])?.[1] ?? 0;
  return end > 0 ? unitFrom(known, body, end) : 0;
};

const nearest = (
  body: string,
  node: Node,
  offset: number,
  known: Reckoned | null,
): number | null => {
  const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
  const block = element?.closest(BLOCK);
  const sheet = element?.closest(".page");
  if (!block) return sheet ? between(body, sheet, node, offset, known) : null;
  const page = block.ownerDocument;
  let before: Element | null = null;
  for (const one of block.querySelectorAll(PLACED)) {
    const start = page.createRange();
    start.setStartBefore(one);
    if (start.comparePoint(node, offset) <= 0) {
      const at = spanOf(one);
      return at ? unitFrom(known, body, at[0]) : null;
    }
    before = one;
  }
  if (before) {
    const at = spanOf(before);
    return at ? unitFrom(known, body, at[1]) : null;
  }
  const at = placeOf(block);
  if (!at) return null;
  // A block with no words yet, as a step just begun, is written before the line break that ends it.
  const start = unitFrom(known, body, at[0]);
  let end = unitFrom(known, body, at[1]);
  while (end > start && /[\r\n]/.test(body[end - 1] ?? "")) end--;
  return end;
};

export const sourceAt = (
  body: string,
  node: Node,
  offset: number,
  known: Reckoned | null = null,
): number | null => {
  const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
  const placed = element?.closest(PLACED);
  if (!placed) return nearest(body, node, offset, known);
  const at = spanOf(placed);
  if (!at) return null;
  if (solid(placed)) return unitFrom(known, body, at[1]);
  const run = pointIn(placed.ownerDocument, placed, 0);
  run.setEnd(node, offset);
  return unitFrom(known, body, at[0]) + run.toString().length;
};

const textAt = (holder: Element, chars: number): [Node, number] => {
  const walk = holder.ownerDocument.createTreeWalker(holder, NodeFilter.SHOW_TEXT);
  let left = chars;
  let last: Text | null = null;
  for (let one = walk.nextNode() as Text | null; one; one = walk.nextNode() as Text | null) {
    if (left <= one.length) return [one, left];
    left -= one.length;
    last = one;
  }
  return last ? [last, last.length] : [holder, holder.childNodes.length];
};

const innermost = (page: Document, byte: number): Element | null => {
  const sheet = page.querySelector(".page");
  let found: Element | null = null;
  let size = Number.POSITIVE_INFINITY;
  for (const top of sheet ? topsAt(sheet, byte) : []) {
    for (const one of [top, ...top.querySelectorAll(BLOCK)]) {
      const at = placeOf(one);
      if (at && at[0] <= byte && byte <= at[1] && at[1] - at[0] <= size) {
        found = one;
        size = at[1] - at[0];
      }
    }
  }
  return found;
};

// Where the caret goes for a place in the words: inside the piece that holds it, or beside the nearest one.
export const spotOf = (
  page: Document,
  body: string,
  chars: number,
  known: Reckoned | null = null,
): [Node, number] | null => {
  const byte = byteFrom(known, body, chars);
  const block = innermost(page, byte);
  if (!block) return null;
  let before: Element | null = null;
  // An item's own line is its own; the words of the list nested in it belong to their items.
  const item = block.matches("li");
  for (const one of block.querySelectorAll(PLACED)) {
    if (item && one.closest(BLOCK) !== block) continue;
    const at = spanOf(one);
    if (!at) continue;
    if (at[0] <= byte && byte <= at[1] && !solid(one))
      return textAt(one, chars - unitFrom(known, body, at[0]));
    if (at[0] >= byte) {
      const parent = one.parentNode;
      if (!parent) return null;
      return before ? endOf(before) : [parent, indexOf(one)];
    }
    before = one;
  }
  if (!before && item) return [block, 0];
  return before ? endOf(before) : [block, block.childNodes.length];
};

const indexOf = (one: Node) => Array.prototype.indexOf.call(one.parentNode?.childNodes ?? [], one);

export const endOf = (one: Element): [Node, number] =>
  solid(one) && one.parentNode
    ? [one.parentNode, indexOf(one) + 1]
    : textAt(one, Number.MAX_SAFE_INTEGER);

// Where a block's own words end, past a piece drawn whole, as a tag at the end of its line.
export const endIn = (block: Element): [Node, number] => {
  const own = Array.from(block.querySelectorAll(PLACED)).filter(
    (one) => one.closest(BLOCK) === block,
  );
  const last = own.pop();
  return last ? endOf(last) : [block, block.childNodes.length];
};

export const GHOST = "[data-ghost]";

// A paragraph just begun holds no words yet, so the core draws nothing there; one stands in until it does.
export const ghostAt = (
  page: Document,
  body: string,
  chars: number,
  known: Reckoned | null = null,
): HTMLElement | null => {
  const sheet = page.querySelector(".page");
  if (!sheet) return null;
  const byte = byteFrom(known, body, chars);
  let after: Element | null = null;
  for (let at = firstReaching(sheet, byte, true) - 1; at >= 0 && !after; at--) {
    if (placeOf(sheet.children[at])) after = sheet.children[at];
  }
  // A page with nothing written yet shows where its title goes, and that is where its first line begins.
  const title = after ? null : sheet.querySelector<HTMLElement>(":scope > h1[data-title]");
  if (title) return title;
  const ghost = page.createElement("p");
  ghost.dataset.ghost = "";
  ghost.append(page.createElement("br"));
  if (after) after.after(ghost);
  else sheet.prepend(ghost);
  return ghost;
};
