import { bytesAt, charsAt, spanned } from "tisty-editor/read";

const PLACED = "[data-src]";
const BLOCK = "[data-at]";

const spanOf = (one: Element) => spanned(one.getAttribute("data-src"));
const solid = (one: Element) => one.getAttribute("contenteditable") === "false";

const pointIn = (page: Document, node: Node, offset: number) => {
  const point = page.createRange();
  point.setStart(node, offset);
  return point;
};

// A point between pieces of words takes the nearest one in its own block, or the block's end when it has none.
// A point on the sheet outside every block, as its margin or a page with nothing yet, writes after the block before it.
const between = (body: string, sheet: Element, node: Node, offset: number): number => {
  const page = sheet.ownerDocument;
  let end = 0;
  for (const one of sheet.querySelectorAll(":scope > [data-at]")) {
    const start = page.createRange();
    start.setStartBefore(one);
    if (start.comparePoint(node, offset) <= 0) break;
    end = spanned(one.getAttribute("data-at"))?.[1] ?? end;
  }
  return end > 0 ? charsAt(body, end) : 0;
};

const nearest = (body: string, node: Node, offset: number): number | null => {
  const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
  const block = element?.closest(BLOCK);
  const sheet = element?.closest(".page");
  if (!block) return sheet ? between(body, sheet, node, offset) : null;
  const page = block.ownerDocument;
  let before: Element | null = null;
  for (const one of block.querySelectorAll(PLACED)) {
    const start = page.createRange();
    start.setStartBefore(one);
    if (start.comparePoint(node, offset) <= 0) {
      const at = spanOf(one);
      return at ? charsAt(body, at[0]) : null;
    }
    before = one;
  }
  const at = before ? spanOf(before) : spanned(block.getAttribute("data-at"));
  return at ? charsAt(body, at[1]) : null;
};

export const sourceAt = (body: string, node: Node, offset: number): number | null => {
  const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
  const placed = element?.closest(PLACED);
  if (!placed) return nearest(body, node, offset);
  const at = spanOf(placed);
  if (!at) return null;
  if (solid(placed)) return charsAt(body, at[1]);
  const run = pointIn(placed.ownerDocument, placed, 0);
  run.setEnd(node, offset);
  return charsAt(body, at[0]) + run.toString().length;
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
  let found: Element | null = null;
  let size = Number.POSITIVE_INFINITY;
  for (const one of page.querySelectorAll(`.page ${BLOCK}`)) {
    const at = spanned(one.getAttribute("data-at"));
    if (at && at[0] <= byte && byte <= at[1] && at[1] - at[0] < size) {
      found = one;
      size = at[1] - at[0];
    }
  }
  return found;
};

// Where the caret goes for a place in the words: inside the piece that holds it, or beside the nearest one.
export const spotOf = (page: Document, body: string, chars: number): [Node, number] | null => {
  const byte = bytesAt(body, chars);
  const block = innermost(page, byte);
  if (!block) return null;
  let before: Element | null = null;
  for (const one of block.querySelectorAll(PLACED)) {
    const at = spanOf(one);
    if (!at) continue;
    if (at[0] <= byte && byte <= at[1] && !solid(one))
      return textAt(one, chars - charsAt(body, at[0]));
    if (at[0] >= byte) {
      const parent = one.parentNode;
      if (!parent) return null;
      return before ? endOf(before) : [parent, indexOf(one)];
    }
    before = one;
  }
  return before ? endOf(before) : [block, block.childNodes.length];
};

const indexOf = (one: Node) => Array.prototype.indexOf.call(one.parentNode?.childNodes ?? [], one);

const endOf = (one: Element): [Node, number] =>
  solid(one) && one.parentNode
    ? [one.parentNode, indexOf(one) + 1]
    : textAt(one, Number.MAX_SAFE_INTEGER);

export const GHOST = "[data-ghost]";

// A paragraph just begun holds no words yet, so the core draws nothing there; one stands in until it does.
export const ghostAt = (page: Document, body: string, chars: number): HTMLElement | null => {
  const sheet = page.querySelector(".page");
  if (!sheet) return null;
  const byte = bytesAt(body, chars);
  let after: Element | null = null;
  for (const one of sheet.querySelectorAll(":scope > [data-at]")) {
    const at = spanned(one.getAttribute("data-at"));
    if (at && at[1] <= byte) after = one;
  }
  const ghost = page.createElement("p");
  ghost.dataset.ghost = "";
  ghost.append(page.createElement("br"));
  if (after) after.after(ghost);
  else sheet.prepend(ghost);
  return ghost;
};
