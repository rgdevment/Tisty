import { charsAt } from "tisty-editor/read";
import { aloneOnLine, EMBED, embedGone, embedPlace } from "./embeds";
import type { Begun } from "./pageBegun";
import { blank, blockPlaced, pastedAround } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { type Span, unsplit } from "./pageEdits";
import { fenceJustOpened, fenceNamed, fenceOpened } from "./pageFences";
import type { Formats } from "./pageFormats";
import type { PageHistory } from "./pageHistory";
import type { Laying } from "./pageLaying";
import {
  closing,
  leftBehind,
  openedAbove,
  opening,
  outdented,
  parted,
  unmarked,
} from "./pageLists";
import { closedRun, unpaired } from "./pageMarks";
import { elementOf, ghostly, inCode, onSheet, sectioned, unwritable } from "./pageNodes";
import { placeOf } from "./pagePlaces";
import type { Writer } from "./pageWriter";

const PAIRS: Record<string, string> = {
  formatBold: "**",
  formatItalic: "*",
  formatStrikeThrough: "~~",
  formatUnderline: "++",
};

// Blocks a key never joins to its neighbour: their marks hold what they are, and the browser's own reach skips them whole.
const SOLID_BLOCK =
  ".table, figure, details, hr, blockquote[data-callout], .front, [contenteditable='false']";

const ATOMS = "[contenteditable='false'], img, button";

// A press on a page break leaves the caret on the sheet beside it, or on its label, never in words.
const breakAt = (range: AbstractRange): Element | null => {
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
const DRESSING = "button, [contenteditable='false']:not([data-src])";

// A paragraph holding nothing but a picture, a card or a formula is drawn whole as well.
const drawnWhole = (one: Element) => {
  if (one.matches(SOLID_BLOCK)) return true;
  const copy = one.cloneNode(true) as Element;
  const atoms = copy.querySelectorAll(ATOMS);
  if (!atoms.length) return false;
  for (const atom of atoms) atom.remove();
  return copy.textContent?.trim() === "";
};

export const inputOn = (
  writer: Writer,
  caret: Caret,
  laying: Laying,
  history: PageHistory,
  formats: Formats,
  begun: Begun,
) => {
  const { page, hands } = writer;
  const { put } = laying;
  let composing: Span | null = null;

  const pasted = (one: Span, data: DataTransfer | null) => {
    const html = data?.getData("text/html") || null;
    const plain = data?.getData("text/plain") ?? "";
    hands
      .translated(html, plain)
      .then((said) => put({ ...one, insert: pastedAround(hands.text(), one.from, one.to, said) }))
      .catch(() => put({ ...one, insert: plain }));
  };

  // A slash opens the blocks where a word could start, as it always did; inside code it is only a slash.
  const slashes = (text: string, one: Span, range: AbstractRange) =>
    one.from === one.to &&
    !inCode(range.startContainer) &&
    (blank(text, one.from) || /(^|\s)$/.test(text.slice(0, one.from).split("\n").pop() ?? ""));

  // A page break holds no words, so the key that would join a block to it takes the break away instead.
  const pastBreak = (kind: string): Span | null => {
    const back = kind === "deleteContentBackward";
    if (!back && kind !== "deleteContentForward") return null;
    const range = caret.chosen();
    if (!range?.collapsed) return null;
    const top = elementOf(range.startContainer)?.closest(".page > [data-at]");
    if (!top) return null;
    const edge = page.createRange();
    edge.selectNodeContents(top);
    if (back) edge.setEnd(range.startContainer, range.startOffset);
    else edge.setStart(range.startContainer, range.startOffset);
    if (edge.toString() !== "") return null;
    const beside = back ? top.previousElementSibling : top.nextElementSibling;
    const shown = beside && aloneOnLine(beside) ? beside.querySelector(EMBED) : null;
    if (shown) return embedGone(hands.text(), shown);
    const at = beside?.matches("hr[data-at]") ? placeOf(beside) : null;
    if (!beside || !at) return null;
    const until = placeOf(beside.nextElementSibling)?.[0] ?? at[1];
    const text = hands.text();
    return { from: charsAt(text, at[0]), to: charsAt(text, until) };
  };

  // A picture or a file among words is drawn whole, so the browser's reach misses it; the key beside it takes it away.
  const pastPicture = (kind: string): Span | null => {
    const back = kind === "deleteContentBackward";
    if (!back && kind !== "deleteContentForward") return null;
    const range = caret.chosen();
    const at = range?.collapsed ? caret.caretSpan()?.from : undefined;
    if (at === undefined) return null;
    const text = hands.text();
    for (const one of page.querySelectorAll(`.page :is(${EMBED})`)) {
      const own = embedPlace(one);
      if (own && charsAt(text, back ? own[1] : own[0]) === at) return embedGone(text, one);
    }
    return null;
  };

  // At a block's edge the key takes nothing when this block or the one beside it is drawn whole, or would leave a fold or a note.
  const edgeHeld = (kind: string) => {
    const back = kind === "deleteContentBackward";
    if (!back && kind !== "deleteContentForward") return false;
    const range = caret.chosen();
    if (!range?.collapsed) return false;
    const inner = elementOf(range.startContainer)?.closest(".page [data-at]");
    if (!inner) return false;
    const edge = page.createRange();
    edge.selectNodeContents(inner);
    if (back) edge.setEnd(range.startContainer, range.startOffset);
    else edge.setStart(range.startContainer, range.startOffset);
    const between = edge.cloneContents();
    for (const one of between.querySelectorAll(DRESSING)) one.remove();
    if (between.textContent !== "") return false;
    if (drawnWhole(inner)) return true;
    const step = (one: Element) => (back ? one.previousElementSibling : one.nextElementSibling);
    for (let at: Element | null = inner; at && !at.matches(".page"); ) {
      let beside = step(at);
      while (beside && !beside.matches("[data-at]")) beside = step(beside);
      if (beside) return drawnWhole(beside);
      at = at.parentElement?.closest(".page [data-at], .page") ?? null;
      if (at?.matches(SOLID_BLOCK)) return true;
    }
    return false;
  };

  const caretFreed = () => {
    const range = caret.chosen();
    const at = caret.caretSpan()?.from;
    if (!range?.collapsed || at === undefined || !elementOf(range.startContainer)?.closest("li"))
      return null;
    return unmarked(hands.text(), at);
  };

  const asked = (event: InputEvent) => {
    if (!onSheet(page, event) || event.isComposing || event.inputType === "insertCompositionText")
      return;
    event.preventDefault();
    const kind = event.inputType;
    writer.leftFrom = undefined;
    if (kind === "insertParagraph") hands.settle?.();
    writer.typing = kind === "insertText" || kind.startsWith("delete");
    if (kind !== "insertText" && !kind.startsWith("format")) writer.pending = null;
    if (kind === "historyUndo") return history.undo();
    if (kind === "historyRedo") return history.redo();
    const broken = pastBreak(kind) ?? pastPicture(kind);
    if (broken) {
      const stays =
        kind === "deleteContentForward" ? (caret.caretSpan()?.from ?? broken.from) : broken.from;
      return put({ ...broken, insert: "" }, Math.min(stays, broken.from));
    }
    if (edgeHeld(kind)) return;
    const [target] = event.getTargetRanges?.() ?? [];
    const range = target ?? caret.chosen();
    const picked = caret.chosen();
    const spread = Boolean(picked && !picked.collapsed);
    const bare = kind === "deleteContentBackward" && !spread ? caretFreed() : null;
    if (bare) return put(bare.edit, bare.caret);
    const shelf = range && !spread ? breakAt(range) : null;
    if (shelf) {
      const span = placeOf(shelf);
      const first = kind === "insertText" ? (event.data ?? "") : "";
      if (span && first) {
        begun.begin(charsAt(hands.text(), span[1]), "", shelf);
        if (writer.kept) begun.firstKey(writer.kept, first);
      }
      return;
    }
    // A caret dropped on a table, a picture or a widget has no words under it to write among.
    if (range && !spread && unwritable(range.startContainer)) return;
    const held = picked ?? range;
    const begunNow =
      writer.kept?.lead !== undefined &&
      (writer.kept.offset < 0 || Boolean(held && ghostly(held.startContainer)));
    if (begunNow && kind.startsWith("delete")) return begun.dropBegun();
    const one =
      (!spread && begun.waited(kind)) ||
      (range &&
        ((held && caret.unbegun(kind, held)) ??
          caret.spanOfRange(range, !spread && kind.startsWith("delete"))));
    if (!range || !one) return;
    const text = hands.text();
    const typed = event.data ?? event.dataTransfer?.getData("text/plain") ?? "";
    const line = begunNow ? writer.kept : null;
    if (line && kind !== "insertText" && kind !== "insertReplacementText") return;
    if (line && kind === "insertText" && typed === "/")
      return hands.blocks(caret.caretPoint(), true);
    if (line) return begun.firstKey(line, typed);
    if (kind === "insertText" && typed === "/" && slashes(text, one, range))
      return hands.blocks(caret.caretPoint(), true);
    if (kind === "insertText" && one.from === one.to) {
      const fence = !inCode(range.startContainer) && fenceOpened(text, one.from, typed);
      if (fence) return put(fence.edit, fence.caret);
      if (writer.pending?.at === one.from) return formats.pendingKey(one.from, typed);
      const opened = text.slice(text.lastIndexOf("\n", one.from - 1) + 1, one.from);
      if (typed === " " && /^\s*(?:>\s?)*\[ ?\]$/.test(opened)) {
        const box = `${opened.replace(/\[ ?\]$/, "")}- [ ] `;
        return put({ from: one.from - opened.length, to: one.from, insert: box });
      }
      // On a task's page a heading is a section of its own, so marks typed for one stay words.
      if (typed === " " && /^#{1,6}$/.test(opened) && sectioned(page)) {
        const from = one.from - opened.length;
        return put({ from, to: one.from, insert: `\\${opened} ` }, one.from + 2);
      }
      if (formats.carriedKey(text, one, typed)) return;
    }
    writer.carried = null;
    if (kind === "insertText" || kind === "insertReplacementText")
      return put({ ...one, insert: typed });
    const blankLine =
      text[one.from - 1] === "\n" &&
      /\r?\n$/.test(text.slice(0, one.from - 1)) &&
      (one.from >= text.length || /[\r\n]/.test(text[one.from] ?? ""));
    if (kind === "insertParagraph" && one.from === one.to && blankLine) return;
    const fold = elementOf(range.startContainer)?.closest(".page details[data-at] > summary")
      ?.parentElement as HTMLDetailsElement | null | undefined;
    if (kind === "insertParagraph" && fold) return begun.intoFold(fold);
    if (kind === "insertParagraph" && one.from === one.to && !inCode(range.startContainer)) {
      const above = openedAbove(text, one.from);
      const block = elementOf(range.startContainer)?.closest("li[data-at], .page > [data-at]");
      if (above && block) return begun.beginAbove(above.at, above.seed, above.tail, block);
      const up = outdented(text, one.from);
      if (up) return put(up.edit, up.caret);
      const left = leftBehind(text, one.from);
      if (left) {
        put({ from: left.from, to: left.to, insert: "" }, left.from);
        const now = hands.text();
        writer.kept = {
          node: page.body,
          offset: -1,
          at: left.from,
          lead: opening(now, left.from, ""),
          tail: closing(now, left.from),
        };
        return;
      }
    }
    if (kind === "insertParagraph" || kind === "insertLineBreak") {
      const coded = inCode(range.startContainer);
      const named =
        kind === "insertParagraph" && coded && one.from === one.to && fenceNamed(text, one.from);
      if (named) return put(named.edit, named.caret);
      if (kind === "insertParagraph" && coded && fenceJustOpened(text, one.from)) return;
      const at = one.from === one.to && !coded ? closedRun(text, one.from) : one.from;
      const insert = kind === "insertLineBreak" ? "\n" : parted(text, at, coded);
      return put({ from: at, to: Math.max(at, one.to), insert });
    }
    if (kind === "insertFromPaste") return pasted(one, event.dataTransfer);
    if (kind.startsWith("delete")) {
      const gone = unpaired(text, unsplit(text, one));
      // A title's mark alone is what is left of a page emptied again, so the page is left empty.
      const left = text.slice(0, gone.from) + text.slice(gone.to);
      if (/^#[ \t]*\r?\n?$/.test(left)) return put({ from: 0, to: text.length, insert: "" }, 0);
      return put({ ...gone, insert: "" }, gone.from);
    }
    const pair = PAIRS[kind];
    if (pair) formats.format(pair, pair);
  };

  const started = (event: CompositionEvent) => {
    if (!onSheet(page, event)) return;
    const range = caret.chosen();
    composing = range && caret.spanOfRange(range);
  };

  const ended = (event: CompositionEvent) => {
    if (!onSheet(page, event)) return;
    const one = composing;
    composing = null;
    writer.typing = true;
    if (writer.kept?.lead !== undefined && one?.from === writer.kept.at)
      return begun.firstKey(writer.kept, event.data ?? "");
    if (one && one.from === one.to && writer.pending?.at === one.from)
      return formats.pendingKey(one.from, event.data ?? "");
    if (one) put({ ...one, insert: event.data ?? "" });
  };

  const pastedFiles = (event: ClipboardEvent) => {
    const files = Array.from(event.clipboardData?.files ?? []);
    const attach = hands.pasted;
    if (
      !onSheet(page, event) ||
      !files.length ||
      !attach ||
      event.clipboardData?.getData("text/plain")
    )
      return;
    event.preventDefault();
    const span = caret.caretSpan();
    if (!span) return;
    const end = Math.max(span.to, caret.blockEnd(span.to));
    attach(files)
      .then((said) => {
        if (!said) return;
        const now = hands.text();
        const at = Math.min(span.from, now.length);
        const laid = blockPlaced(now, at, Math.min(end, now.length), said, said.length);
        put(laid.edit, laid.caret);
        // The words typed next go under the file just laid, never into its line.
        const after = hands.text();
        const under = laid.edit.from + laid.edit.insert.replace(/\s+$/, "").length;
        writer.kept = {
          node: page.body,
          offset: -1,
          at: under,
          lead: opening(after, under, ""),
          tail: closing(after, under),
        };
      })
      .catch(() => {});
  };

  return { asked, started, ended, pastedFiles };
};
