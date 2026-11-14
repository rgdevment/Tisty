import { SLASHED } from "tisty-editor/edit";
import type { Beside } from "./pageBeside";
import type { Caret } from "./pageCaret";
import type { Formats } from "./pageFormats";
import type { PageHistory } from "./pageHistory";
import type { Laying } from "./pageLaying";
import { closing, nested, opening } from "./pageLists";
import { swappedItem } from "./pageMoving";
import { elementOf, inCode, onSheet, WRITABLE } from "./pageNodes";
import { endIn } from "./pageSpots";
import type { TagList } from "./pageTags";
import type { Writer } from "./pageWriter";

const KEYED: Record<string, readonly [string, string]> = {
  b: ["**", "**"],
  i: ["*", "*"],
  u: ["++", "++"],
  "shift x": ["~~", "~~"],
  "shift c": ["`", "`"],
  k: ["[", "]()"],
};

export const keysOn = (
  writer: Writer,
  caret: Caret,
  laying: Laying,
  history: PageHistory,
  formats: Formats,
  beside: Beside,
  tags: TagList,
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
    const laid = beside.slashed(rule);
    if (!laid) return;
    const text = hands.text();
    let at =
      writer.kept && writer.kept.offset < 0
        ? writer.kept.at
        : Math.min(text.length, laid.from + laid.insert.replace(/\s+$/, "").length);
    while (at > 0 && /[\r\n]/.test(text[at - 1] ?? "")) at--;
    writer.kept = {
      node: page.body,
      offset: -1,
      at,
      lead: opening(text, at, ""),
      tail: closing(text, at),
    };
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

  // A tag or other piece drawn whole at a line's end leaves the browser no place after it, so End is placed here.
  const toEnd = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey) {
      const last = Array.from(page.querySelectorAll(".page [data-at]"))
        .filter((one) => one.closest(WRITABLE) && !one.closest('[contenteditable="false"]'))
        .pop();
      if (!last) return;
      event.preventDefault();
      caret.laidAt(endIn(last));
      return;
    }
    const from = caret.blockOfCaret();
    if (!from) return;
    setTimeout(() => {
      if (caret.blockOfCaret() !== from && from.isConnected) caret.laidAt(endIn(from));
    });
  };

  const keyed = (event: KeyboardEvent) => {
    if (!onSheet(page, event)) return;
    if (event.key === "End" && !event.shiftKey && !event.altKey) return toEnd(event);
    if (event.key === "Tab" && !event.ctrlKey && !event.metaKey && !event.altKey)
      return tabbed(event);
    if (event.altKey && (event.key === "ArrowUp" || event.key === "ArrowDown"))
      return stepped(event);
    if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
    if (event.key === "Enter") return broke(event);
    const named = `${event.shiftKey ? "shift " : ""}${event.key.toLowerCase()}`;
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
    if (caret.blockStart() !== writer.leftFrom) hands.settle?.();
    writer.leftFrom = undefined;
  };

  const ticking = (event: MouseEvent) => {
    if ((event.target as Element | null)?.closest?.(".page .check, .page input[type=checkbox]"))
      event.preventDefault();
  };

  // The page's events come from its own window, so a key is told by what it carries, not by its class.
  const moved = (event: Event) => {
    const key = "key" in event ? String(event.key) : null;
    if (key !== null && !key.startsWith("Arrow") && !/^(Home|End|Page)/.test(key)) return;
    if ((event.target as Element | null)?.closest?.(".tag-list")) return;
    const held = event as KeyboardEvent;
    const across = key !== null && Boolean(held.ctrlKey || held.metaKey);
    if (key === null || /^(ArrowUp|ArrowDown|Page)/.test(key) || across)
      writer.leftFrom = caret.blockStart();
    writer.kept = null;
    writer.pending = null;
    writer.carried = null;
    writer.typing = false;
    tags.hide();
  };

  return { keyed, noted, ticking, moved };
};
