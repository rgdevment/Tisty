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
  opening = false,
): number | null => {
  const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
  const placed = element?.closest(PLACED);
  if (!placed) return nearest(body, node, offset, known);
  const at = spanOf(placed);
  if (!at) return null;
  // A reach that opens inside a piece that cannot be written in, as a formula's drawing, takes the piece whole.
  if (solid(placed)) return unitFrom(known, body, opening ? at[0] : at[1]);
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

const blankAfter = (body: string, chars: number) => {
  let end = chars;
  while (body[end] === " " || body[end] === "\t") end++;
  return end - chars;
};

const wordsAhead = (page: Document, body: string, chars: number, known: Reckoned | null) => {
  const byte = byteFrom(known, body, chars);
  const block = innermost(page, byte);
  const blank = block ? 0 : blankAfter(body, chars);
  const ahead = blank ? innermost(page, byte + blank) : null;
  return ahead
    ? { chars: chars + blank, byte: byte + blank, block: ahead }
    : { chars, byte, block };
};

// Where the caret goes for a place in the words: inside the piece that holds it, or beside the nearest one.
export const spotOf = (
  page: Document,
  body: string,
  asked: number,
  known: Reckoned | null = null,
): [Node, number] | null => {
  const { chars, byte, block } = wordsAhead(page, body, asked, known);
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
  if (!before && item) return [block, boxEnd(block)];
  return before ? endOf(before) : [block, block.childNodes.length];
};

export const indexOf = (one: Node) =>
  Array.prototype.indexOf.call(one.parentNode?.childNodes ?? [], one);

// A step with no words yet is written after its box, never before it.
export const boxEnd = (item: Element) => {
  const box = item.querySelector(":scope > .check");
  return box ? indexOf(box) + 1 : 0;
};

export const endOf = (one: Element): [Node, number] =>
  solid(one) && one.parentNode
    ? [one.parentNode, indexOf(one) + 1]
    : textAt(one, Number.MAX_SAFE_INTEGER);

export const startOf = (one: Element): [Node, number] =>
  solid(one) && one.parentNode ? [one.parentNode, indexOf(one)] : textAt(one, 0);

const shut = (node: Node | undefined) =>
  node?.nodeType === Node.ELEMENT_NODE &&
  solid(node as Element) &&
  !(node as Element).matches("br");

const caretStands = ([node, offset]: [Node, number], toStart: boolean) =>
  node.nodeType === Node.TEXT_NODE || !shut(node.childNodes[toStart ? offset - 1 : offset]);

const ownOf = (block: Element, selector: string) =>
  Array.from(block.querySelectorAll(selector)).filter((one) => one.closest(BLOCK) === block);

// Where a block's own words end, past a piece drawn whole, as a tag at the end of its line.
export const endIn = (block: Element): [Node, number] => {
  const last = ownOf(block, PLACED).pop();
  return last ? endOf(last) : [block, block.childNodes.length];
};

export const startIn = (block: Element): [Node, number] => {
  const first = block.querySelector(PLACED);
  return first ? startOf(first) : [block, 0];
};

type Spot = [Node, number];

const STOP = `${PLACED}, br`;

const deepest = (node: Node) => {
  let one = node;
  while (one.lastChild) one = one.lastChild;
  return one;
};

const stopOf = (node: Node | null | undefined) =>
  node?.nodeType === Node.ELEMENT_NODE && (node as Element).matches(`${STOP}, ${BLOCK}`)
    ? (node as Element)
    : null;

const wordsOf = (one: Element, cut: Spot | null, toStart: boolean) => {
  if (!cut) return one.textContent ?? "";
  const part = one.ownerDocument.createRange();
  part.selectNodeContents(one);
  if (toStart) part.setEnd(...cut);
  else part.setStart(...cut);
  return part.toString();
};

const walkStart = (node: Node, offset: number, toStart: boolean): [Element | null, Node] => {
  if (node.nodeType !== Node.ELEMENT_NODE) return [null, node];
  const beside = node.childNodes[toStart ? offset - 1 : offset];
  if (!beside) return [null, toStart ? node : deepest(node)];
  const near = toStart ? deepest(beside) : beside;
  const first = stopOf(beside) ?? (toStart ? stopOf(near) : null);
  if (!first) return [null, near];
  return [first, toStart ? first : deepest(first)];
};

function* stopsFrom(block: Element, from: Spot, toStart: boolean) {
  const walk = block.ownerDocument.createTreeWalker(block, NodeFilter.SHOW_ELEMENT, {
    acceptNode: (one) =>
      one !== block && stopOf(one) ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_SKIP,
  });
  const [node, offset] = from;
  const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
  const holder = element?.closest(STOP);
  if (holder && block.contains(holder)) {
    yield [holder, from] as const;
    walk.currentNode = toStart ? holder : deepest(holder);
  } else {
    const [first, start] = walkStart(node, offset, toStart);
    walk.currentNode = start;
    if (first) yield [first, null] as const;
  }
  const step = () => (toStart ? walk.previousNode() : walk.nextNode()) as Element | null;
  for (let one = step(); one; one = step()) yield [one, null] as const;
}

function* ownStops(block: Element, from: Spot, toStart: boolean) {
  for (const [one, cut] of stopsFrom(block, from, toStart)) {
    if (one.matches("br") || one.closest(BLOCK) !== block) return;
    yield [one, () => wordsOf(one, cut, toStart)] as const;
  }
}

const lineOf = (block: Element, from: Spot, toStart: boolean) => {
  let last: Element | null = null;
  let trail: Element[] = [];
  for (const [one, words] of ownStops(block, from, toStart)) {
    if (words().includes("\n")) return null;
    last = one;
    if (solid(one)) trail.push(one);
    else trail = [one];
  }
  const spots = trail.map((one) => (toStart ? startOf(one) : endOf(one)));
  const edge = spots.reverse().find((spot) => caretStands(spot, toStart)) ?? from;
  return { last, edge };
};

const letterBox = ([node, offset]: Spot, toStart: boolean) => {
  const at = toStart ? offset : offset - 1;
  if (node.nodeType !== Node.TEXT_NODE || at < 0 || at >= (node as Text).length) return null;
  const letter = (node.ownerDocument ?? document).createRange();
  letter.setStart(node, at);
  letter.setEnd(node, at + 1);
  const box =
    typeof letter.getBoundingClientRect === "function" ? letter.getBoundingClientRect() : null;
  return box?.height ? box : null;
};

const otherLine = (landed: Spot, toStart: boolean) => {
  const line = letterBox(landed, toStart);
  return (one: Element) => {
    if (!line) return false;
    const box = one.getBoundingClientRect();
    const middle = (box.top + box.bottom) / 2;
    return toStart ? middle < line.top : middle > line.bottom;
  };
};

const wordsUntil = (
  block: Element,
  from: Spot,
  last: Element,
  toStart: boolean,
  apart: (one: Element) => boolean,
) => {
  for (const [one, words] of ownStops(block, from, toStart)) {
    if (solid(one) ? apart(one) : words().trim()) return true;
    if (one === last) return false;
  }
  return false;
};

export const lineEdge = (
  block: Element,
  from: Spot,
  landed: Spot,
  toStart: boolean,
  visual = false,
): Spot | null => {
  const [node, offset] = from;
  const room = node.nodeType === Node.TEXT_NODE ? (node as Text).length : node.childNodes.length;
  if (!block.contains(node) || offset > room) return null;
  const line = lineOf(block, from, toStart);
  if (!line) return null;
  const page = block.ownerDocument;
  const { last } = line;
  const bound = !last ? from : toStart ? startOf(last) : endOf(last);
  const within = page.createRange();
  within.setStart(...(toStart ? bound : from));
  within.setEnd(...(toStart ? from : bound));
  if (!block.contains(landed[0]) || !within.isPointInRange(...landed)) return line.edge;
  const apart = visual ? otherLine(landed, toStart) : () => false;
  if (last && wordsUntil(block, landed, last, toStart, apart)) return null;
  const edge = page.createRange();
  edge.setStart(...line.edge);
  const side = edge.comparePoint(...landed);
  const outer = toStart ? side <= 0 : side >= 0;
  return outer && caretStands(landed, toStart) ? null : line.edge;
};

export const GHOST = "[data-ghost]";

// The core marks the paragraph right after the properties as the title, so a line begun there wears it too.
export const titledAfterFront = (ghost: Element) => {
  const after = ghost.previousElementSibling?.matches(
    ".page > .front:first-child, .page > .front:first-child + .front.agent-own",
  );
  if (after && ghost.matches("p") && !ghost.nextElementSibling?.matches(".title"))
    ghost.classList.add("title");
};

// A block whose words end at the caret stands before it, though its line breaks reach past.
const endsAt = (body: string, chars: number, byte: number, block: Element | undefined) => {
  const end = block && placeOf(block)?.[1];
  return end !== undefined && end > byte && /^[\r\n]*$/.test(body.slice(chars, chars + end - byte));
};

// A paragraph just begun holds no words yet, so the core draws nothing there; one stands in until it does.
export const ghostAt = (
  page: Document,
  body: string,
  chars: number,
  known: Reckoned | null = null,
  lead = "",
): HTMLElement | null => {
  const sheet = page.querySelector(".page");
  if (!sheet) return null;
  const byte = byteFrom(known, body, chars);
  let reach = firstReaching(sheet, byte, true);
  while (reach < sheet.children.length && !placeOf(sheet.children[reach])) reach++;
  if (endsAt(body, chars, byte, sheet.children[reach])) reach++;
  let after: Element | null = null;
  for (let at = reach - 1; at >= 0 && !after; at--) {
    if (placeOf(sheet.children[at])) after = sheet.children[at];
  }
  // A page with nothing written yet shows where its title goes, and that is where its first line begins.
  const title = after ? null : sheet.querySelector<HTMLElement>(":scope > h1[data-title]");
  if (title) return title;
  const ghost = page.createElement("p");
  ghost.dataset.ghost = "";
  ghost.append(page.createElement("br"));
  const button = after?.nextElementSibling;
  const step = /\[ \]\s*$/.test(lead);
  // A list's «+ Agregar un paso» hangs on the list, so only a new step is drawn above it.
  if (button?.matches(".add-step") && !step) button.after(ghost);
  else if (after) after.after(ghost);
  else sheet.prepend(ghost);
  titledAfterFront(ghost);
  return ghost;
};
