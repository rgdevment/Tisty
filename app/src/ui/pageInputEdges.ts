import type { Span } from "./pageEdits";
import { foldReached } from "./pageFolds";
import { closingOnly, wordsFrom } from "./pageMarks";
import {
  ATOMS,
  elementOf,
  emptyOfWords,
  inCode,
  PIECES,
  shownWords,
  wholeBehind,
  withoutDressing,
  wordsEndAt,
  wordsStartAt,
} from "./pageNodes";
import { endIn, indexOf, ownsWords, startIn, startOf } from "./pageSpots";
import { keptUnlaid, type Writer } from "./pageWriter";

// Blocks a key never joins to its neighbour: their marks hold what they are, and the browser's own reach skips them whole.
const SOLID_BLOCK =
  ".table, figure, .math, details, hr, blockquote[data-callout], .front, [contenteditable='false']";

export const HEADINGS = "h1, h2, h3, h4, h5, h6";
export const MARKED_LINE = `${HEADINGS}, blockquote:not([data-callout])`;
export const TITLE_PLACE = ".page > [data-at]:not(.front)";

export const backward = (kind: string) => kind.startsWith("delete") && kind.endsWith("Backward");
export const forward = (kind: string) => kind.startsWith("delete") && kind.endsWith("Forward");

// A press on a page break leaves the caret on the sheet beside it, or on its label, never in words.
export const breakAt = (range: AbstractRange): Element | null => {
  const node = range.startContainer;
  const element = elementOf(node);
  const sheet = element?.closest(".page");
  if (!element || !sheet) return null;
  const said = element.closest(".break-said");
  if (said) {
    const nth = Array.from(sheet.querySelectorAll(":scope > .break-said")).indexOf(said);
    return sheet.querySelectorAll(":scope > hr[data-at]")[nth] ?? null;
  }
  const own = element.closest(".page > hr[data-at]");
  if (own || element !== sheet) return own;
  const beside = [node.childNodes[range.startOffset], node.childNodes[range.startOffset - 1]];
  return (
    (beside.find((one) => one instanceof Element && one.matches("hr[data-at]")) as Element) ?? null
  );
};

export const nothingPast = (edge: Range, back: boolean) => {
  const rest = shownWords(withoutDressing(edge).textContent ?? "");
  return rest === "" || (!back && rest === "\n" && inCode(edge.startContainer));
};

const TAKEN_WHOLE = `${PIECES}, br, hr, input`;

const holdsSomething = (one: Node) =>
  one.nodeType === Node.TEXT_NODE
    ? shownWords((one as Text).data) !== ""
    : (one as Element).matches(TAKEN_WHOLE);

const pastSubtree = (one: Node, root: Node): Node | null => {
  let at: Node | null = one;
  while (at && at !== root && !at.nextSibling) at = at.parentNode;
  return at && at !== root ? at.nextSibling : null;
};

// A caret at the very edge of the words, or in a block drawn empty, leaves the browser's own delete nothing to take.
export const nothingToTake = (node: Node, offset: number, back: boolean) => {
  const sheet = elementOf(node)?.closest(".page");
  if (!sheet) return false;
  const text = node.nodeType === Node.TEXT_NODE;
  const length = text ? (node as Text).length : node.childNodes.length;
  if (text && (back ? offset > 0 : offset < length)) return false;
  const walk = sheet.ownerDocument.createTreeWalker(
    sheet,
    NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT,
  );
  let next: Node | null;
  if (back) {
    walk.currentNode = !text && offset > 0 ? node.childNodes[offset - 1] : node;
    if (!text && offset > 0) while (walk.lastChild());
    next = !text && offset > 0 ? walk.currentNode : walk.previousNode();
  } else {
    next = !text && offset < length ? node.childNodes[offset] : pastSubtree(node, sheet);
    if (next) walk.currentNode = next;
  }
  while (next && next !== sheet) {
    if (holdsSomething(next)) return false;
    next = back ? walk.previousNode() : walk.nextNode();
  }
  return true;
};

export const titleAloneLeft = (was: string, left: string) =>
  /^#[ \t]*\r?\n?$/.test(left) || (/^#[ \t]/.test(was) && !left.trim());

export const inOneCodeBlock = (range: AbstractRange) => {
  const block = elementOf(range.startContainer)?.closest("pre");
  return Boolean(block) && block === elementOf(range.endContainer)?.closest("pre");
};

export const elementIn = (node: Node | undefined) =>
  node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : null;

export const wordsBeside = (range: AbstractRange, back: boolean) => {
  const node = elementOf(range.startContainer)?.childNodes[range.startOffset - (back ? 1 : 0)];
  return node !== undefined && !elementIn(node)?.matches(PIECES);
};

export const blockOf = (node: Node) => elementOf(node)?.closest(".page [data-at]");

const ownWordsEndAt = (block: Element, node: Node, offset: number) => {
  const rest = block.ownerDocument.createRange();
  rest.setStart(node, offset);
  rest.setEnd(...endIn(block));
  return emptyOfWords(rest);
};

export const cutAtEdge = (named: StaticRange, picked: AbstractRange | null, kind: string) => {
  if (!kind.startsWith("deleteWord") || !picked?.collapsed) return named;
  const back = backward(kind);
  const [ownEdge, besideEdge] = back ? [wordsStartAt, wordsEndAt] : [ownWordsEndAt, wordsStartAt];
  const reached: [Node, number] = back
    ? [named.startContainer, named.startOffset]
    : [named.endContainer, named.endOffset];
  const own = blockOf(picked.startContainer);
  const beside = blockOf(reached[0]);
  if (!own || !beside || own === beside) return named;
  if (!ownEdge(own, picked.startContainer, picked.startOffset) || besideEdge(beside, ...reached))
    return named;
  const cut = own.ownerDocument.createRange();
  if (back) {
    cut.setStart(...endIn(beside));
    cut.setEnd(named.endContainer, named.endOffset);
  } else {
    cut.setStart(named.startContainer, named.startOffset);
    cut.setEnd(...startOf(beside));
  }
  return cut;
};

export const joined = (text: string, one: Span, range: AbstractRange): Span | null => {
  const ending = blockOf(range.endContainer);
  const starting = blockOf(range.startContainer);
  if (range.collapsed || !ending || !starting) return null;
  if (!wordsStartAt(ending, range.endContainer, range.endOffset)) return null;
  if (!wordsEndAt(starting, range.startContainer, range.startOffset)) return null;
  const to = Math.max(one.from, wordsFrom(text, one.to));
  const lineEnd = text.indexOf("\n", one.from);
  const closed = lineEnd >= 0 && closingOnly(text.slice(one.from, lineEnd));
  return { from: closed ? Math.min(lineEnd, to) : one.from, to };
};

export const blockBeside = (inner: Element, back: boolean): Element | null => {
  const step = (one: Element) => (back ? one.previousElementSibling : one.nextElementSibling);
  for (let at: Element | null = inner; at && !at.matches(".page"); ) {
    let beside = step(at);
    while (beside && !beside.matches("[data-at]")) beside = step(beside);
    if (beside) return beside;
    at = at.parentElement?.closest(".page [data-at], .page") ?? null;
    if (at?.matches(SOLID_BLOCK)) return at;
  }
  return null;
};

// A paragraph holding nothing but a picture, a card or a formula is drawn whole as well.
export const drawnWhole = (one: Element) => {
  if (one.matches(SOLID_BLOCK)) return true;
  const copy = one.cloneNode(true) as Element;
  const atoms = copy.querySelectorAll(ATOMS);
  if (!atoms.length) return false;
  for (const atom of atoms) atom.remove();
  return shownWords(copy.textContent ?? "").trim() === "";
};

export const nestedAfter = (node: Node, offset: number): Element | null => {
  const item = blockOf(node);
  const nested = item?.matches("li")
    ? Array.from(item.children).find((one) => one.matches("[data-at]"))
    : null;
  if (!item || !nested) return null;
  const gap = item.ownerDocument.createRange();
  gap.setStart(node, offset);
  if (gap.comparePoint(item, indexOf(nested)) < 0) return null;
  gap.setEnd(item, indexOf(nested));
  return emptyOfWords(gap) ? nested : null;
};

const wordsOpen = (block: Element) => {
  const line = blockOf(startIn(block)[0]);
  return Boolean(line && ownsWords(line) && !drawnWhole(line));
};

export const keptApart = (nested: Element) =>
  nested.matches(SOLID_BLOCK) || (nested.matches("blockquote") && !wordsOpen(nested));

export const onlyInto = (range: AbstractRange, nested: Element) => {
  const { endContainer, endOffset } = range;
  const atStart = endContainer === nested.parentNode && endOffset === indexOf(nested);
  if (!atStart && !nested.contains(endContainer)) return false;
  const chosen = nested.ownerDocument.createRange();
  chosen.setStart(range.startContainer, range.startOffset);
  chosen.setEnd(endContainer, endOffset);
  return emptyOfWords(chosen);
};

export const beforeFeeds = (text: string, one: Span): Span => {
  let to = one.to;
  while (to > one.from && /[\r\n]/.test(text[to - 1] ?? "")) to--;
  return { from: one.from, to };
};

export const pastWhole = (range: AbstractRange) => {
  if (wholeBehind(range.endContainer, range.endOffset)) return true;
  const block = elementIn(range.endContainer.childNodes[range.endOffset - 1]);
  const atSheet = elementIn(range.endContainer)?.matches(".page");
  return Boolean(atSheet && block && drawnWhole(block)) || foldReached(range) !== null;
};

export const leftOver = (writer: Writer, picked: AbstractRange | null) => {
  const left = writer.leftChosen;
  return (
    keptUnlaid(writer.kept) &&
    left !== null &&
    picked !== null &&
    picked.startContainer === left.from[0] &&
    picked.startOffset === left.from[1] &&
    picked.endContainer === left.to[0] &&
    picked.endOffset === left.to[1]
  );
};
