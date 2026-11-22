import { charsAt } from "tisty-editor/read";
import { byteFrom, unitFrom } from "../placing";
import { aloneOnLine, EMBED, embedGone, embedPlace, spacedOut } from "./embeds";
import type { Begun } from "./pageBegun";
import { blank, blockPlaced, pastedAround } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { lineEnd, reachIn, type Span, unsplit } from "./pageEdits";
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
import { closedRun, closingOnly, escapeTaken, keptGone, replaced, wordsFrom } from "./pageMarks";
import { atGhost, elementOf, inCode, onSheet, sectioned, unwritable } from "./pageNodes";
import { placeOf } from "./pagePlaces";
import { compositionOpened, type Writer } from "./pageWriter";

const PAIRS: Record<string, string> = {
  formatBold: "**",
  formatItalic: "*",
  formatStrikeThrough: "~~",
  formatUnderline: "++",
};

// Blocks a key never joins to its neighbour: their marks hold what they are, and the browser's own reach skips them whole.
const SOLID_BLOCK =
  ".table, figure, details, hr, blockquote[data-callout], .front, [contenteditable='false']";

const PIECES = "[contenteditable='false'], img";
const ATOMS = `${PIECES}, button`;

const backward = (kind: string) => kind.startsWith("delete") && kind.endsWith("Backward");
const forward = (kind: string) => kind.startsWith("delete") && kind.endsWith("Forward");

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

const undressed = (range: Range) => {
  const between = range.cloneContents();
  for (const one of between.querySelectorAll(DRESSING)) one.remove();
  return between;
};

const sideOf = (holder: Element, node: Node, offset: number, back: boolean) => {
  const side = holder.ownerDocument.createRange();
  side.selectNodeContents(holder);
  if (back) side.setEnd(node, offset);
  else side.setStart(node, offset);
  return side;
};

const emptyOfWords = (range: Range) => {
  const between = undressed(range);
  return between.textContent === "" && !between.querySelector(ATOMS);
};

const wordsStartAt = (holder: Element, node: Node, offset: number) =>
  emptyOfWords(sideOf(holder, node, offset, true));

const wordsEndAt = (holder: Element, node: Node, offset: number) =>
  emptyOfWords(sideOf(holder, node, offset, false));

const inOneCodeBlock = (range: AbstractRange) => {
  const block = elementOf(range.startContainer)?.closest("pre");
  return Boolean(block) && block === elementOf(range.endContainer)?.closest("pre");
};

const elementIn = (node: Node | undefined) =>
  node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : null;

const wordsBeside = (range: AbstractRange, back: boolean) => {
  const node = elementOf(range.startContainer)?.childNodes[range.startOffset - (back ? 1 : 0)];
  return node !== undefined && !elementIn(node)?.matches(PIECES);
};

const blockOf = (node: Node) => elementOf(node)?.closest(".page [data-at]");

const joined = (text: string, one: Span, range: AbstractRange): Span | null => {
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

const blockBeside = (inner: Element, back: boolean): Element | null => {
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
  let composing: { readonly one: Span; readonly range: Range; readonly was: string } | null = null;

  const putOver = (one: Span, insert: string, coded: boolean) => {
    if (coded) return put({ ...one, insert });
    const laid = replaced(hands.text(), one, insert);
    put(laid.edit, laid.caret);
  };

  const pasted = (one: Span, data: DataTransfer | null, coded: boolean) => {
    const html = data?.getData("text/html") || null;
    const plain = data?.getData("text/plain") ?? "";
    hands
      .translated(html, plain)
      .then((said) => putOver(one, pastedAround(hands.text(), one.from, one.to, said), coded))
      .catch(() => putOver(one, plain, coded));
  };

  // A slash opens the blocks where a word could start, as it always did; inside code it is only a slash.
  const slashes = (text: string, one: Span, range: AbstractRange) =>
    one.from === one.to &&
    !inCode(range.startContainer) &&
    (blank(text, one.from) || /(^|\s)$/.test(text.slice(0, one.from).split("\n").pop() ?? ""));

  // A page break holds no words, so the key that would join a block to it takes the break away instead.
  const pastBreak = (kind: string): Span | null => {
    const back = backward(kind);
    if (!back && !forward(kind)) return null;
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
  const pictureBeside = (kind: string, at: number): Span | null => {
    const back = backward(kind);
    if (!back && !forward(kind)) return null;
    const text = hands.text();
    const byte = byteFrom(writer.reckoned, text, at);
    for (const one of page.querySelectorAll(`.page :is(${EMBED})`)) {
      const own = embedPlace(one);
      if (own && (back ? own[1] : own[0]) === byte) return embedGone(text, one);
    }
    return null;
  };

  const pastPicture = (kind: string): Span | null => {
    const range = caret.chosen();
    const at = range?.collapsed ? caret.caretSpan()?.from : undefined;
    return at === undefined ? null : pictureBeside(kind, at);
  };

  // At a block's edge the key takes nothing when this block or the one beside it is drawn whole, or would leave a fold or a note.
  const edgeHeld = (kind: string) => {
    const back = backward(kind);
    if (!back && !forward(kind)) return false;
    const range = caret.chosen();
    if (!range?.collapsed) return false;
    const inner = elementOf(range.startContainer)?.closest(".page [data-at]");
    if (!inner) return false;
    const edge = sideOf(inner, range.startContainer, range.startOffset, back);
    if (undressed(edge).textContent !== "") return false;
    if (drawnWhole(inner)) return true;
    const beside = blockBeside(inner, back);
    return beside !== null && drawnWhole(beside);
  };

  const joinedAfter = (at: number): Span | null => {
    const text = hands.text();
    const end = lineEnd(text, at);
    const held = caret.chosen();
    const inner = held ? blockOf(held.startContainer) : null;
    const after = inner ? blockBeside(inner, false) : null;
    const place = after && inner && !after.contains(inner) ? placeOf(after) : null;
    if (!place || !closingOnly(text.slice(at, end))) return null;
    const next = unitFrom(writer.reckoned, text, place[0]);
    if (next <= end || text.slice(end, next).trim()) return null;
    return { from: end, to: wordsFrom(text, lineEnd(text, next)) };
  };

  const caretKept = (kind: string, from: number) =>
    forward(kind) ? Math.min(caret.caretSpan()?.from ?? from, from) : from;

  const tookBeside = (kind: string, gone: Span) => {
    const { edit } = replaced(hands.text(), gone, "");
    put(edit, caretKept(kind, edit.from));
  };

  const besideGone = (kind: string) => pastBreak(kind) ?? pastPicture(kind);

  const pieceBeside = (range: AbstractRange, back: boolean) => {
    const holder = range.collapsed ? elementOf(range.startContainer) : null;
    if (holder !== range.startContainer || !holder?.closest(".page [data-at]")) return null;
    const step = back ? -1 : 1;
    let at = range.startOffset - (back ? 1 : 0);
    while (elementIn(holder.childNodes[at])?.matches("button")) at += step;
    const piece = elementIn(holder.childNodes[at]);
    return piece?.matches(PIECES) ? piece : null;
  };

  const pieceGone = (piece: Element): Span | null => {
    const text = hands.text();
    if (piece.matches(EMBED)) return embedGone(text, piece);
    const own = placeOf(piece, "data-src");
    if (!own) return null;
    return spacedOut(
      text,
      unitFrom(writer.reckoned, text, own[0]),
      unitFrom(writer.reckoned, text, own[1]),
    );
  };

  const leftOver = (picked: AbstractRange | null) => {
    const left = writer.leftChosen;
    const unlaid = writer.kept !== null && writer.kept.offset < 0 && writer.kept.lead === undefined;
    return (
      unlaid &&
      left !== null &&
      picked !== null &&
      picked.startContainer === left.from[0] &&
      picked.startOffset === left.from[1] &&
      picked.endContainer === left.to[0] &&
      picked.endOffset === left.to[1]
    );
  };

  const caretFreed = () => {
    const range = caret.chosen();
    const at = caret.caretSpan()?.from;
    const item = elementOf(range?.startContainer)?.closest("li");
    if (!range?.collapsed || at === undefined || !item) return null;
    const text = hands.text();
    const first = wordsStartAt(item, range.startContainer, range.startOffset);
    return unmarked(text, first ? wordsFrom(text, at) : at);
  };

  const typedIn = (one: Span, range: AbstractRange, typed: string) => {
    const text = hands.text();
    if (typed === "/" && slashes(text, one, range)) return hands.blocks(caret.caretPoint(), true);
    if (one.from === one.to) {
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
    putOver(one, typed, inOneCodeBlock(range));
  };

  const asked = (event: InputEvent) => {
    if (!onSheet(page, event) || event.isComposing || event.inputType === "insertCompositionText")
      return;
    event.preventDefault();
    const kind = event.inputType;
    const broken = besideGone(kind);
    if (!broken && edgeHeld(kind)) return;
    writer.leftFrom = undefined;
    if (kind === "insertParagraph") hands.settle?.();
    writer.typing = kind === "insertText" || kind.startsWith("delete");
    if (kind !== "insertText" && !kind.startsWith("format")) writer.pending = null;
    if (kind === "historyUndo") return history.undo();
    if (kind === "historyRedo") return history.redo();
    if (broken) return tookBeside(kind, broken);
    const [named] = event.getTargetRanges?.() ?? [];
    const picked = caret.chosen();
    const target = named && !(named.collapsed && picked?.collapsed) ? named : undefined;
    const range = target ?? picked;
    const spread = !leftOver(picked) && Boolean(picked && !picked.collapsed);
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
      (writer.kept.offset < 0 || Boolean(held && atGhost(held.startContainer, held.startOffset)));
    if (begunNow && kind.startsWith("delete")) return begun.dropBegun();
    const reached =
      (!spread && begun.waited(kind)) ||
      (range &&
        ((held && caret.unbegun(kind, held)) ??
          caret.spanOfRange(range, !spread && kind.startsWith("delete"))));
    if (!range || !reached) return;
    const text = hands.text();
    const missed = !spread && reached.from === reached.to && (backward(kind) || forward(kind));
    const behind = missed && target && backward(kind) ? pictureBeside(kind, reached.from) : null;
    if (behind) return tookBeside(kind, behind);
    const blind = missed && (forward(kind) || !target);
    const unshown = !missed && target && !spread && kind === "deleteContentForward";
    const at = blind || unshown ? caret.caretSpan()?.from : undefined;
    const skipped = unshown && at !== undefined && /^[ \t]+$/.test(text.slice(at, reached.from));
    const ahead = blind || skipped ? at : undefined;
    const own = ahead === undefined ? null : reachIn(text, ahead, kind);
    const inLine =
      own !== null &&
      own.from >= wordsFrom(text, own.to) &&
      !text.slice(own.from, own.to).includes("\n");
    const reach = own && inLine ? own : reached;
    const ended = blind && forward(kind) && ahead !== undefined ? joinedAfter(ahead) : null;
    const joining = joined(text, reach, range) ?? ended;
    const kept = joining ?? reach;
    const coded = inCode(range.startContainer);
    const literal = inOneCodeBlock(range);
    const one = coded ? kept : escapeTaken(text, kept);
    const typed = event.data ?? event.dataTransfer?.getData("text/plain") ?? "";
    const line = begunNow ? writer.kept : null;
    if (line && kind !== "insertText" && kind !== "insertReplacementText") return;
    if (line && kind === "insertText" && typed === "/")
      return hands.blocks(caret.caretPoint(), true);
    if (line) return begun.firstKey(line, typed);
    if (kind === "insertText") return typedIn(one, range, typed);
    writer.carried = null;
    if (kind === "insertReplacementText") return putOver(one, typed, literal);
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
      const named =
        kind === "insertParagraph" && coded && one.from === one.to && fenceNamed(text, one.from);
      if (named) return put(named.edit, named.caret);
      if (kind === "insertParagraph" && coded && fenceJustOpened(text, one.from)) return;
      const at = one.from === one.to && !coded ? closedRun(text, one.from) : one.from;
      const insert = kind === "insertLineBreak" ? "\n" : parted(text, at, coded);
      return put({ from: at, to: Math.max(at, one.to), insert });
    }
    if (kind === "insertFromPaste") return pasted(one, event.dataTransfer, literal);
    if (kind.startsWith("delete")) {
      const span = unsplit(text, one);
      const keyed =
        !joining && !spread && !kind.includes("Line") && (backward(kind) || forward(kind));
      const reachAt = (from: number) => reachIn(text, from, kind);
      const key = keyed ? { back: backward(kind), reach: reachAt } : null;
      const gone = literal ? { ...span, insert: "" } : keptGone(text, span, key);
      // A title's mark alone is what is left of a page emptied again, so the page is left empty.
      const left = text.slice(0, gone.from) + gone.insert + text.slice(gone.to);
      if (/^#[ \t]*\r?\n?$/.test(left)) return put({ from: 0, to: text.length, insert: "" }, 0);
      return put(gone, spread ? gone.from : caretKept(kind, gone.from));
    }
    const pair = PAIRS[kind];
    if (pair) formats.format(pair, pair);
  };

  const started = (event: CompositionEvent) => {
    if (!onSheet(page, event)) return;
    writer.composing ??= compositionOpened();
    const held = page.getSelection();
    const range = held?.rangeCount ? held.getRangeAt(0).cloneRange() : null;
    const spread = Boolean(range && !range.collapsed && !leftOver(range));
    const one = range && ((!spread && begun.waited("insertText")) || caret.spanOfRange(range));
    composing = range && one ? { one, range, was: hands.text() } : null;
  };

  const ended = (event: CompositionEvent) => {
    writer.composing?.end();
    writer.composing = null;
    const held = composing;
    composing = null;
    if (!onSheet(page, event)) return;
    writer.typing = true;
    const typed = event.data ?? "";
    if (!held) return;
    if (!typed) {
      const same = held.was === hands.text();
      if (same && held.one.from < held.one.to) putOver(held.one, "", inOneCodeBlock(held.range));
      return;
    }
    const kept = writer.kept;
    const { startContainer, startOffset } = held.range;
    if (kept?.lead !== undefined && (kept.offset < 0 || atGhost(startContainer, startOffset)))
      return typed === "/" ? hands.blocks(caret.caretPoint(), true) : begun.firstKey(kept, typed);
    typedIn(held.one, held.range, typed);
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

  const markHeld = (event: KeyboardEvent) => {
    const back = event.key === "Backspace";
    if ((!back && event.key !== "Delete") || event.defaultPrevented || event.isComposing) return;
    if (!onSheet(page, event) || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey)
      return;
    const range = caret.chosen();
    const at = caret.caretSpan()?.from;
    if (!range || at === undefined) return;
    const amongWords = range.startContainer.nodeType === Node.TEXT_NODE;
    if (amongWords && (!back || hands.text().lastIndexOf("\n", at - 1) >= 0)) return;
    const kind = back ? "deleteContentBackward" : "deleteContentForward";
    const piece = pieceBeside(range, back);
    const gone = besideGone(kind) ?? (piece && pieceGone(piece));
    const held = !gone && edgeHeld(kind);
    const boxed = !back && Boolean(piece?.matches(".check"));
    const freed = gone || held || (!back && !boxed) ? null : caretFreed();
    const acted = gone || freed || held;
    if (!acted && (!piece || wordsBeside(range, !back))) return;
    event.preventDefault();
    if (!gone && !freed) return;
    writer.leftFrom = undefined;
    writer.typing = true;
    writer.pending = null;
    if (freed) return put(freed.edit, freed.caret);
    if (gone) tookBeside(kind, gone);
  };

  return { asked, started, ended, pastedFiles, markHeld };
};
