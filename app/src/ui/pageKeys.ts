import { SLASHED } from "tisty-editor/edit";
import { bytesAt, charsAt } from "tisty-editor/read";
import {
  caretWalk,
  type Edge,
  edgeWalk,
  type LineKey,
  letterOf,
  lineKey,
  modKey,
  moveKey,
  pageReach,
} from "./macKeys";
import type { Beside } from "./pageBeside";
import type { Caret } from "./pageCaret";
import type { Choosing } from "./pageChoosing";
import { colonsKept } from "./pageColons";
import type { Formats } from "./pageFormats";
import type { PageHistory } from "./pageHistory";
import type { Laying } from "./pageLaying";
import { nested } from "./pageLists";
import { balanced } from "./pageMarks";
import { swappedItem } from "./pageMoving";
import {
  besideBox,
  brokenBy,
  caretLaidOn,
  DRAWN_BLOCK,
  elementOf,
  inCode,
  letterAhead,
  onSheet,
  pieceEnding,
  SOLID_PIECE,
  shownWords,
  unseenLetter,
  WHOLE,
  WRITABLE,
} from "./pageNodes";
import {
  endIn,
  endOf,
  indexOf,
  lineEdge,
  ownsWords,
  type Spot,
  sourceAt,
  spotOf,
  startIn,
  startOf,
} from "./pageSpots";
import { keptUnlaid, type Writer } from "./pageWriter";

const KEYED: Record<string, readonly [string, string]> = {
  b: ["**", "**"],
  i: ["*", "*"],
  u: ["++", "++"],
  "shift x": ["~~", "~~"],
  "shift c": ["`", "`"],
  k: ["[", "]()"],
};

type Way = "ArrowLeft" | "ArrowRight" | "forward" | "backward";

const AHEAD = /^(ArrowRight|ArrowDown|End|PageDown)$/;
const BACK = /^(ArrowLeft|ArrowUp|Home|PageUp)$/;

const wayOf = (key: string, walk: string | null) => {
  if (walk !== null) return "fnev".includes(walk) ? 1 : -1;
  return AHEAD.test(key) ? 1 : BACK.test(key) ? -1 : 0;
};

const EMPTY_STEP = /^[ \t]*(?:>[ \t]?)*[-*+][ \t]+\[ \][ \t]*\r?$/;

type EdgeAwaited = {
  readonly laid: (inView: boolean) => void;
  readonly timer: ReturnType<typeof setTimeout>;
};

const FOCUS_KEPT =
  ".page .check, .page input[type=checkbox], .page [contenteditable='false'] a[href^='#']";

export const keysOn = (
  writer: Writer,
  caret: Caret,
  laying: Laying,
  history: PageHistory,
  formats: Formats,
  beside: Beside,
  tags: { readonly hide: () => void },
  choosing: Choosing,
) => {
  const { page, hands } = writer;
  const { put } = laying;

  // Alt with an arrow moves the item or the block the caret stands in, as it did in the Markdown.
  const stepped = (event: KeyboardEvent) => {
    const way = event.key === "ArrowUp" ? -1 : 1;
    const element = elementOf(page.getSelection()?.anchorNode ?? null);
    const sheet = page.querySelector(".page");
    if (!element || !sheet) return;
    event.preventDefault();
    const item = element.closest("li[data-at]");
    if (item && sheet.contains(item)) return beside.replaced(swappedItem(hands.text(), item, way));
    const top = element.closest(".page > [data-at]");
    if (top) beside.unitActed(top, way < 0 ? "up" : "down");
  };

  const broke = (event: KeyboardEvent) => {
    event.preventDefault();
    hands.settle?.();
    const range = caret.chosen();
    const rule = SLASHED.find((one) => one.key === "rule");
    if (!range || !rule || inCode(range.startContainer)) return;
    beside.slashed(rule);
  };

  // Tab nests a list item under the one above and Shift+Tab takes it back out; in code it indents.
  const tabbed = (event: KeyboardEvent) => {
    event.preventDefault();
    const at = caret.caretSpan()?.from;
    const range = caret.chosen();
    if (at === undefined || !range) return;
    if (inCode(range.startContainer)) {
      if (!event.shiftKey) put({ from: at, to: at, insert: "  " });
      return;
    }
    const moved = nested(hands.text(), at, event.shiftKey);
    if (moved) put(moved.edit, moved.caret);
  };

  let edgeAwaited: EdgeAwaited | null = null;

  const edgeLetGo = () => {
    const awaited = edgeAwaited;
    edgeAwaited = null;
    if (awaited) clearTimeout(awaited.timer);
    return awaited;
  };

  const edgeLaid = () => edgeLetGo()?.laid(true);

  const edgeClicked = () => edgeLetGo()?.laid(false);

  const afterMove = (laid: (inView: boolean) => void) => {
    edgeLaid();
    edgeAwaited = { laid, timer: setTimeout(edgeLaid) };
  };

  const toLineEdge = ({ toStart, visual }: Edge, outOnly = false) => {
    edgeLaid();
    const range = caret.chosen();
    if (!range) return;
    const from: [Node, number] = toStart
      ? [range.startContainer, range.startOffset]
      : [range.endContainer, range.endOffset];
    const block = elementOf(from[0])?.closest(".page [data-at]");
    if (!block) return;
    const span = caret.spanOfRange(range);
    const at = toStart ? span?.from : span?.to;
    const left = caret.blockStart();
    writer.leftFrom = undefined;
    const lineNow = (): [Element, [Node, number]] | null => {
      if (block.isConnected) return [block, from];
      const spot = at === undefined ? null : spotOf(page, hands.text(), at, writer.reckoned);
      const again = spot && elementOf(spot[0])?.closest(".page [data-at]");
      return spot && again ? [again, spot] : null;
    };
    const corrected = (inView: boolean) => {
      const held = caret.chosen();
      const line = lineNow();
      if (!held || !line || (outOnly && caret.blockOfCaret() === line[0])) return;
      const landed: [Node, number] = [held.startContainer, held.startOffset];
      const spot = lineEdge(...line, landed, toStart, visual) ?? (outOnly ? endIn(line[0]) : null);
      if (!spot || (held.startContainer === spot[0] && held.startOffset === spot[1])) return;
      if (inView) caret.laidAt(spot);
      else caretLaidOn(page, ...spot);
    };
    afterMove((inView) => {
      corrected(inView);
      leftBehind(left);
    });
  };

  const toLineKey = ({ toStart, reaching }: LineKey) => {
    if (!reaching) toLineEdge({ toStart, visual: true });
    const way = toStart ? "backward" : "forward";
    page.getSelection()?.modify(reaching ? "extend" : "move", way, "lineboundary");
  };

  const sideOfBox = () => {
    const range = caret.chosen();
    return range?.collapsed ? besideBox(range.startContainer, range.startOffset) : null;
  };

  let keptBehind = false;
  let lastWay = 0;

  const outOfMark = (range: AbstractRange) => {
    const ahead = letterAhead(range.startContainer, range.startOffset);
    if (!ahead || ahead[0] === range.startContainer) return false;
    caretLaidOn(page, ...ahead);
    return true;
  };

  const outOfLetter = (range: AbstractRange) => {
    const letter = range.startContainer;
    const piece = letter.previousSibling;
    if (!unseenLetter(letter) || piece?.nodeType !== Node.ELEMENT_NODE || !letter.parentNode)
      return false;
    const beyond = brokenBy(piece as Element);
    const onward = beyond && range.startOffset > 0 && lastWay > 0;
    const before: Spot = [letter.parentNode, indexOf(letter)];
    caretLaidOn(page, ...(onward ? lineAfter(beyond) : before));
    return true;
  };

  const brokenNear = (node: Node) => {
    const block = elementOf(node)?.closest("[data-at]");
    const outer = block?.parentElement?.closest("li[data-at]");
    return [block, outer].flatMap((one) =>
      one ? Array.from(one.children).filter((piece) => brokenBy(piece)) : [],
    );
  };

  const lineAfter = (beyond: Element): Spot => {
    if (beyond.matches("br")) return endOf(beyond);
    if (!beyond.matches('[contenteditable="false"]')) return startIn(beyond);
    const next = writableBlocks().find(
      (one) => beyond.compareDocumentPosition(one) & Node.DOCUMENT_POSITION_FOLLOWING,
    );
    return next ? startIn(next) : endOf(beyond);
  };

  const aheadOf = (way: Way, holder: Element) => {
    if (way === "forward" || way === "backward") return way === "forward";
    const rtl = page.defaultView?.getComputedStyle(holder).direction === "rtl";
    return (way === "ArrowRight") !== rtl;
  };

  const nothingBetween = (from: Spot, to: Spot, until?: Spot) => {
    const gap = page.createRange();
    gap.setStart(...from);
    if (gap.comparePoint(...to) < 0) return false;
    if (until) gap.setEnd(...until);
    if (until && gap.comparePoint(...to) > 0) return false;
    gap.setEnd(...to);
    if (shownWords(gap.toString()) !== "") return false;
    return !gap.cloneContents().querySelector(`${SOLID_PIECE}, img, br, [data-at]`);
  };

  const sameBlock = (one: Spot, other: Spot) =>
    elementOf(one[0])?.closest("[data-at]") === elementOf(other[0])?.closest("[data-at]");

  const acrossBreak = (way: Way, reaching: boolean): Spot | null => {
    const held = page.getSelection();
    const focus = held?.focusNode;
    if (keptBehind || !focus || !held || (!reaching && !held.isCollapsed)) return null;
    const pieces = brokenNear(focus);
    const holder = elementOf(focus);
    if (!pieces.length || !holder) return null;
    const ahead = aheadOf(way, holder);
    const at: Spot = [focus, held.focusOffset];
    for (const piece of pieces) {
      const beyond = brokenBy(piece);
      const ending = pieceEnding(piece);
      if (!beyond || !ending || !piece.parentNode) continue;
      const before = startOf(ending);
      const after: Spot = [piece.parentNode, indexOf(piece) + 1];
      const broken: Spot = [piece.parentNode, indexOf(beyond)];
      const next = lineAfter(beyond);
      if (ahead && sameBlock(at, before) && nothingBetween(at, before)) return after;
      if (ahead && nothingBetween(after, at, broken)) return next;
      if (!ahead && sameBlock(at, next) && nothingBetween(next, at)) return after;
      if (!ahead && nothingBetween(after, at, broken)) return before;
    }
    return null;
  };

  const pastBox = (key: string) => {
    const held = caret.chosen();
    if (!held?.collapsed || !page.querySelector(".page li.task")) return;
    const back = key === "ArrowLeft";
    const inStep = back && elementOf(held.startContainer)?.closest("li.task");
    const from = inStep ? caret.caretSpan()?.from : undefined;
    afterMove(() => {
      for (let tries = 0; tries < 2; tries++) {
        const side = sideOfBox();
        const unmoved = side === "after" && from !== undefined && caret.caretSpan()?.from === from;
        if (side !== "before" && !unmoved) return;
        page.getSelection()?.modify("move", back ? "left" : "right", "character");
      }
      const range = caret.chosen();
      if (range && sideOfBox() === "before")
        caretLaidOn(page, range.startContainer, range.startOffset + 1);
    });
  };

  const writableBlocks = () =>
    Array.from(page.querySelectorAll(".page [data-at]")).filter(
      (one) => one.closest(WRITABLE) && !one.closest('[contenteditable="false"]'),
    );

  const pageEdges = () => {
    const blocks = writableBlocks();
    return [blocks[0], blocks[blocks.length - 1]] as const;
  };

  const lastBlock = () => pageEdges()[1];

  const wholeBeyond = (block: Element, fromEnd: boolean) => {
    const wholes = Array.from(page.querySelectorAll(WHOLE)).filter(
      (one) => !one.parentElement?.closest('[contenteditable="false"]'),
    );
    const edge = fromEnd ? wholes[wholes.length - 1] : wholes[0];
    const side = fromEnd ? Node.DOCUMENT_POSITION_FOLLOWING : Node.DOCUMENT_POSITION_PRECEDING;
    return edge?.parentNode && block.compareDocumentPosition(edge) & side ? edge : null;
  };

  const pageStart = (first: Element): [Node, number] => {
    const top = wholeBeyond(first, false);
    return top?.parentNode ? [top.parentNode, indexOf(top)] : startIn(first);
  };

  const pageEnd = (last: Element): [Node, number] => {
    const top = wholeBeyond(last, true);
    return top?.parentNode ? [top.parentNode, indexOf(top) + 1] : endIn(last);
  };

  // A tag or other piece drawn whole at a line's end leaves the browser no place after it, so End is placed here.
  const toEnd = (event: KeyboardEvent) => {
    if (modKey(event)) {
      if (event.shiftKey) return;
      const last = lastBlock();
      if (!last) return;
      event.preventDefault();
      caret.laidAt(endIn(last));
      return;
    }
    if (!event.shiftKey && !event.ctrlKey && !event.metaKey)
      toLineEdge({ toStart: false, visual: true }, true);
  };

  const reachedPage = (event: KeyboardEvent, toStart: boolean) => {
    const [first, last] = pageEdges();
    const edge = toStart ? first : last;
    if (!edge) return;
    event.preventDefault();
    caret.reachedTo(toStart ? pageStart(edge) : pageEnd(edge));
  };

  const choseAll = (event: KeyboardEvent) => {
    const [first, last] = pageEdges();
    if (!first || !last) return;
    event.preventDefault();
    page.getSelection()?.setBaseAndExtent(...pageStart(first), ...pageEnd(last));
  };

  const sideReach = (event: KeyboardEvent) =>
    (event.key === "ArrowLeft" || event.key === "ArrowRight") &&
    event.shiftKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.altKey &&
    !event.isComposing;

  const sideArrow = (event: KeyboardEvent) =>
    (event.key === "ArrowLeft" || event.key === "ArrowRight") &&
    !event.shiftKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.altKey &&
    !event.isComposing;

  const takenOut = () => {
    const picked = caret.chosenWhole();
    const one = picked && balanced(hands.text(), picked);
    if (!one) return false;
    const gone = colonsKept(hands.text(), { ...one, insert: "" });
    put(gone, gone.from);
    return true;
  };

  const opensBeforeWhole = () => {
    const range = caret.chosen();
    if (!range || range.collapsed) return false;
    const top = range.startContainer.childNodes[range.startOffset];
    return top?.nodeType === Node.ELEMENT_NODE && (top as Element).matches(WHOLE);
  };

  const outOfSolids = (event: KeyboardEvent) => {
    keptBehind = keptUnlaid(writer.kept);
    if (event.isComposing || !onSheet(page, event)) return;
    if (event.key !== "Backspace" && event.key !== "Delete") return;
    const held = event.shiftKey || event.ctrlKey || event.metaKey || event.altKey;
    if (held || !opensBeforeWhole()) return caret.solidsLeft();
    writer.leftFrom = undefined;
    writer.typing = true;
    writer.pending = null;
    if (takenOut()) event.preventDefault();
  };

  const keyed = (event: KeyboardEvent) => {
    if (!onSheet(page, event)) return;
    if (event.key === "Escape") {
      if (!event.defaultPrevented && !leftHere() && writer.kept?.lead === undefined)
        choosing.choose(event);
      return;
    }
    const line = lineKey(event);
    if (line) {
      event.preventDefault();
      return toLineKey(line);
    }
    const reach = pageReach(event);
    if (reach) return reachedPage(event, reach === "start");
    if (event.key === "End" && !event.altKey) return toEnd(event);
    const edge = edgeWalk(event);
    if (edge) return toLineEdge(edge);
    if (event.key === "Tab" && !event.ctrlKey && !event.metaKey && !event.altKey)
      return tabbed(event);
    if (moveKey(event) && (event.key === "ArrowUp" || event.key === "ArrowDown"))
      return stepped(event);
    const walked = caretWalk(event) === "letter" && !event.shiftKey;
    if (walked || sideArrow(event) || sideReach(event)) {
      const arrow = event.key === "ArrowRight" ? "ArrowRight" : "ArrowLeft";
      const way: Way = walked ? (letterOf(event) === "f" ? "forward" : "backward") : arrow;
      const across = acrossBreak(way, event.shiftKey);
      if (!across) return walked || event.shiftKey ? undefined : pastBox(event.key);
      event.preventDefault();
      if (event.shiftKey) return caret.reachedTo(across);
      const line = elementOf(across[0])?.closest("[data-at]");
      const empty = line && !ownsWords(line) && !line.matches(DRAWN_BLOCK);
      const at = empty ? sourceAt(hands.text(), ...across, writer.reckoned) : null;
      return at === null ? caret.laidAt(across) : caret.placeAt(at);
    }
    if (!modKey(event) || event.altKey) return;
    if (event.key === "Enter") return broke(event);
    const named = `${event.shiftKey ? "shift " : ""}${letterOf(event)}`;
    if (named === "a") return choseAll(event);
    if (named === "z" || named === "shift z" || named === "y") {
      event.preventDefault();
      writer.pending = null;
      writer.carried = null;
      return named === "z" ? history.undo() : history.redo();
    }
    const pair = KEYED[named];
    if (pair) {
      event.preventDefault();
      return formats.format(pair[0], pair[1]);
    }
    if (event.key !== "/") return;
    event.preventDefault();
    hands.blocks(caret.caretPoint(), false);
  };

  // A step left with no words is no step, so its empty box is taken away instead of kept in the file.
  const leftEmpty = (from: string | null): boolean => {
    if (from === null) return false;
    const text = hands.text();
    const start = charsAt(text, Number(from));
    const ends = text.indexOf("\n", start);
    const line = text.slice(start, ends < 0 ? text.length : ends);
    if (!EMPTY_STEP.test(line)) return false;
    const gone = ends < 0 ? line.length : line.length + 1;
    const begun = writer.kept?.lead !== undefined ? writer.kept : null;
    const now = begun?.at ?? caret.caretSpan()?.from ?? start;
    const shifted = (at: number) => (at > start ? Math.max(start, at - gone) : at);
    put({ from: start, to: start + gone, insert: "" }, shifted(now));
    if (begun) writer.kept = { ...begun, at: shifted(begun.at) };
    return true;
  };

  const leftBehind = (from: string | null) => {
    if (caret.blockStart() === from) return;
    leftEmpty(from);
    hands.settle?.();
  };

  // A step left empty with Escape goes, as it goes when the caret leaves its line.
  const leftHere = () => {
    const at = caret.caretSpan()?.from;
    if (at === undefined) return false;
    const text = hands.text();
    return leftEmpty(String(bytesAt(text, text.lastIndexOf("\n", at - 1) + 1)));
  };

  const noted = () => {
    beside.besideLine();
    const range = caret.chosen();
    const sheet = page.querySelector(".page");
    if (range && !range.collapsed && sheet?.contains(range.startContainer))
      writer.lastChosen = caret.spanOfRange(range);
    else if (range?.collapsed) {
      writer.lastChosen = null;
      if (!outOfLetter(range) && !outOfMark(range)) caret.outOfSolid(range);
    }
    if (writer.leftFrom === undefined) return;
    const from = writer.leftFrom;
    writer.leftFrom = undefined;
    leftBehind(from);
  };

  const ticking = (event: MouseEvent) => {
    if ((event.target as Element | null)?.closest?.(FOCUS_KEPT)) event.preventDefault();
  };

  // The page's events come from its own window, so a key is told by what it carries, not by its class.
  const moved = (event: Event) => {
    const key = "key" in event ? String(event.key) : null;
    const held = event as KeyboardEvent;
    const walked = key === null || held.isComposing ? null : caretWalk(held);
    lastWay = key === null ? 0 : wayOf(key, walked ? letterOf(held) : null);
    if (key !== null && !walked && !key.startsWith("Arrow") && !/^(Home|End|Page)/.test(key))
      return;
    if ((event.target as Element | null)?.closest?.(".tag-list")) return;
    const across = key !== null && !walked && Boolean(held.ctrlKey || held.metaKey);
    if (key === null || /^(ArrowUp|ArrowDown|Page)/.test(key) || across || walked === "line")
      writer.leftFrom = caret.blockStart();
    writer.kept = null;
    writer.pending = null;
    writer.carried = null;
    writer.typing = false;
    tags.hide();
  };

  return {
    keyed,
    noted,
    ticking,
    moved,
    outOfSolids,
    takenOut,
    edgeLaid,
    edgeClicked,
    edgeLetGo,
  };
};
