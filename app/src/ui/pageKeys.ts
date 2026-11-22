import { SLASHED } from "tisty-editor/edit";
import { bytesAt, charsAt } from "tisty-editor/read";
import { caretWalk, type Edge, edgeWalk, letterOf, modKey, moveKey } from "./macKeys";
import type { Beside } from "./pageBeside";
import type { Caret } from "./pageCaret";
import type { Choosing } from "./pageChoosing";
import type { Formats } from "./pageFormats";
import type { PageHistory } from "./pageHistory";
import type { Laying } from "./pageLaying";
import { nested } from "./pageLists";
import { swappedItem } from "./pageMoving";
import { elementOf, inCode, onSheet, WRITABLE } from "./pageNodes";
import { endIn, lineEdge, spotOf } from "./pageSpots";
import type { Writer } from "./pageWriter";

const KEYED: Record<string, readonly [string, string]> = {
  b: ["**", "**"],
  i: ["*", "*"],
  u: ["++", "++"],
  "shift x": ["~~", "~~"],
  "shift c": ["`", "`"],
  k: ["[", "]()"],
};

const EMPTY_STEP = /^[ \t]*(?:>[ \t]?)*[-*+][ \t]+\[ \][ \t]*\r?$/;

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

  const toLineEdge = ({ toStart, visual }: Edge, outOnly = false) => {
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
    const corrected = () => {
      const held = caret.chosen();
      const line = lineNow();
      if (!held || !line || (outOnly && caret.blockOfCaret() === line[0])) return;
      const landed: [Node, number] = [held.startContainer, held.startOffset];
      const spot = lineEdge(...line, landed, toStart, visual) ?? (outOnly ? endIn(line[0]) : null);
      if (spot && (held.startContainer !== spot[0] || held.startOffset !== spot[1]))
        caret.laidAt(spot);
    };
    setTimeout(() => {
      corrected();
      leftBehind(left);
    });
  };

  const lastBlock = () =>
    Array.from(page.querySelectorAll(".page [data-at]"))
      .filter((one) => one.closest(WRITABLE) && !one.closest('[contenteditable="false"]'))
      .pop();

  const pageEnd = (last: Element): [Node, number] => {
    const sheet = page.querySelector(".page");
    const tops = sheet?.querySelectorAll(":scope > [data-at]");
    const top = tops?.[tops.length - 1];
    if (!sheet || !top || top.contains(last)) return endIn(last);
    return [sheet, Array.prototype.indexOf.call(sheet.childNodes, top) + 1];
  };

  // A tag or other piece drawn whole at a line's end leaves the browser no place after it, so End is placed here.
  const toEnd = (event: KeyboardEvent) => {
    if (modKey(event)) {
      const last = lastBlock();
      if (!last) return;
      event.preventDefault();
      if (event.shiftKey) caret.reachedTo(pageEnd(last));
      else caret.laidAt(endIn(last));
      return;
    }
    if (!event.shiftKey && !event.ctrlKey && !event.metaKey)
      toLineEdge({ toStart: false, visual: true }, true);
  };

  const keyed = (event: KeyboardEvent) => {
    if (!onSheet(page, event)) return;
    if (event.key === "Escape") {
      if (!event.defaultPrevented && !leftHere() && writer.kept?.lead === undefined)
        choosing.choose(event);
      return;
    }
    if (event.key === "End" && !event.altKey) return toEnd(event);
    const edge = edgeWalk(event);
    if (edge) return toLineEdge(edge);
    if (event.key === "Tab" && !event.ctrlKey && !event.metaKey && !event.altKey)
      return tabbed(event);
    if (moveKey(event) && (event.key === "ArrowUp" || event.key === "ArrowDown"))
      return stepped(event);
    if (!modKey(event) || event.altKey) return;
    if (event.key === "Enter") return broke(event);
    const named = `${event.shiftKey ? "shift " : ""}${letterOf(event)}`;
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
      caret.outOfSolid(range);
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

  return { keyed, noted, ticking, moved };
};
