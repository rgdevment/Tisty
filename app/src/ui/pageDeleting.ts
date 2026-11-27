import { charsAt } from "tisty-editor/read";
import { byteFrom, unitFrom } from "../placing";
import { aloneOnLine, EMBED, embedGone, embedPlace, spacedOut } from "./embeds";
import type { Caret, Standing } from "./pageCaret";
import {
  applied,
  blankAt,
  breakKept,
  hardBroken,
  lineEnd,
  reachIn,
  type Span,
  unsplit,
} from "./pageEdits";
import {
  backward,
  blockBeside,
  blockOf,
  drawnWhole,
  elementIn,
  forward,
  HEADINGS,
  keptApart,
  leftOver,
  MARKED_LINE,
  nestedAfter,
  nothingPast,
  onlyInto,
  pastWhole,
  TITLE_PLACE,
  titleAloneLeft,
  wordsBeside,
} from "./pageInputEdges";
import { mendedOver, type Putting } from "./pageLaidOver";
import type { Laying } from "./pageLaying";
import { unlined, unmarked } from "./pageLists";
import { closingOnly, joinedAcross, keptGone, replaced, wordsFrom } from "./pageMarks";
import type { Mend } from "./pageMends";
import {
  elementOf,
  letterAhead,
  PIECES,
  sectioned,
  shownWords,
  sideOf,
  unseenLetter,
  wordsStartAt,
} from "./pageNodes";
import { placeOf } from "./pagePlaces";
import { startIn } from "./pageSpots";
import { wholeReach } from "./pageWholes";
import { choiceKept, keptUnlaid, titleBegun, type Writer } from "./pageWriter";

// Code and a note keep words of their own to write in; a table, a diagram, a fold or a sticky note go whole.
const GOES_WHOLE = ".table, figure:not(.code), .math, details, hr, blockquote[data-callout='idea']";
// A line of only a card or a formula set apart is a piece of its own; tags, icons and inline pieces are words.
const ALONE = "a.card, .doc-card, math[display='block']";

const aloneAsPiece = (line: Element) =>
  line.matches("p") &&
  drawnWhole(line) &&
  line.querySelectorAll(PIECES).length === 1 &&
  line.querySelector(ALONE) !== null;

const goesWhole = (one: Element) => one.matches(`[data-at]:is(${GOES_WHOLE})`) || aloneAsPiece(one);

const placedBeside = (one: Element, back: boolean) => {
  const step = (at: Element) => (back ? at.previousElementSibling : at.nextElementSibling);
  let beside = step(one);
  while (beside && !beside.matches("[data-at]")) beside = step(beside);
  return beside;
};

export type Retaken = (kind: string, named: StaticRange | undefined, standing: Standing) => void;

export type KeyReach =
  | { readonly gone: Span }
  | { readonly reach: Span; readonly ended: Span | null };

export type Deleted = {
  readonly joining: boolean;
  readonly spread: boolean;
  readonly literal: boolean;
  readonly mend: Mend | null;
};

export const deletingOn = (
  writer: Writer,
  caret: Caret,
  laying: Laying,
  { put }: Putting,
  retaken: Retaken,
) => {
  const { page, hands } = writer;

  // A piece drawn whole holds no words to join, so the key that would join a block to it takes the piece away instead.
  const pastBreak = (kind: string, range: AbstractRange | null): Span | null => {
    const back = backward(kind);
    if (!back && !forward(kind)) return null;
    if (!range?.collapsed) return null;
    const top = elementOf(range.startContainer)?.closest(".page > [data-at]");
    if (!top) return null;
    const edge = page.createRange();
    edge.selectNodeContents(top);
    if (back) edge.setEnd(range.startContainer, range.startOffset);
    else edge.setStart(range.startContainer, range.startOffset);
    if (shownWords(edge.toString()) !== "") return null;
    const beside = placedBeside(top, back);
    const shown = beside && aloneOnLine(beside) ? beside.querySelector(EMBED) : null;
    if (shown) return embedGone(hands.text(), shown);
    const at = beside && goesWhole(beside) ? placeOf(beside) : null;
    if (!beside || !at) return null;
    const text = hands.text();
    const after = placeOf(placedBeside(beside, false));
    if (after) return { from: charsAt(text, at[0]), to: charsAt(text, after[0]) };
    // The last piece of a page takes the blank line above it too, so the page ends at its last words.
    const from = charsAt(text, at[0]);
    return { from: text.slice(0, from).trimEnd().length, to: charsAt(text, at[1]) };
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

  const pastPicture = (kind: string, range: AbstractRange | null): Span | null => {
    const kept = !range && keptUnlaid(writer.kept);
    const at = kept || range?.collapsed ? caret.caretSpan()?.from : undefined;
    return at === undefined ? null : pictureBeside(kind, at);
  };

  // At a block's edge the key takes nothing when this block or the one beside it is drawn whole, or would leave a fold or a note.
  const edgeHeld = (kind: string, range: AbstractRange | null) => {
    const back = backward(kind);
    if (!back && !forward(kind)) return false;
    if (!range) return false;
    const reaching = !back || !range.collapsed;
    const nested = reaching ? nestedAfter(range.startContainer, range.startOffset) : null;
    if (nested && (range.collapsed || onlyInto(range, nested))) return keptApart(nested);
    if (!range.collapsed) return false;
    const inner = elementOf(range.startContainer)?.closest(".page [data-at]");
    if (!inner) return false;
    const edge = sideOf(inner, range.startContainer, range.startOffset, back);
    if (!nothingPast(edge, back)) return false;
    const section = (one: Element) => sectioned(page) && one.matches(HEADINGS);
    if (drawnWhole(inner) || section(inner)) return true;
    const beside = blockBeside(inner, back);
    const freed = back && Boolean(inner.closest("li, blockquote"));
    return beside !== null && (drawnWhole(beside) || (section(beside) && !freed));
  };

  const joinedAfter = (at: number, held: AbstractRange | null): Span | null => {
    const text = hands.text();
    const end = lineEnd(text, at);
    const inner = held ? blockOf(held.startContainer) : null;
    const after = inner ? blockBeside(inner, false) : null;
    const place = after && inner && !after.contains(inner) ? placeOf(after) : null;
    if (!place || !closingOnly(text.slice(at, end))) return null;
    const next = unitFrom(writer.reckoned, text, place[0]);
    if (next <= end || text.slice(end, next).trim()) return null;
    return { from: end, to: wordsFrom(text, lineEnd(text, next)) };
  };

  const unplaced = (kind: string, { range, lagging }: Standing) => {
    const at = writer.kept?.at;
    if (!lagging || range || at === undefined || choiceKept(writer.kept)) return false;
    const text = hands.text();
    if (blankAt(text, at)) return kind !== "deleteContentBackward" || !breakKept(text, at);
    return joinedAcross(text, at, backward(kind)) !== null;
  };

  const caretKept = (kind: string, from: number) =>
    forward(kind) ? Math.min(caret.caretSpan()?.from ?? from, from) : from;

  const emptied = (text: string) => {
    put({ from: 0, to: text.length, insert: "" }, 0, true);
    const begun = titleBegun(page);
    if (begun) writer.kept = { node: page.body, offset: -1, at: 0, ...begun };
  };

  const tookBeside = (kind: string, gone: Span) => {
    const { edit } = replaced(hands.text(), gone, "");
    put(edit, caretKept(kind, edit.from));
  };

  const besideGone = (kind: string, range: AbstractRange | null) =>
    pastBreak(kind, range) ?? pastPicture(kind, range);

  const besideCaret = (range: AbstractRange, back: boolean) => {
    const holder = range.collapsed ? elementOf(range.startContainer) : null;
    if (holder !== range.startContainer || !holder?.closest(".page [data-at]")) return null;
    const step = back ? -1 : 1;
    let at = range.startOffset - (back ? 1 : 0);
    while (
      elementIn(holder.childNodes[at])?.matches("button") ||
      unseenLetter(holder.childNodes[at])
    )
      at += step;
    return elementIn(holder.childNodes[at]);
  };

  const nestedJoined = (range: AbstractRange, beside: Element | null) => {
    if (!beside?.matches("ul, ol, blockquote") || keptApart(beside)) return null;
    const [endContainer, endOffset] = startIn(beside);
    const { startContainer, startOffset } = range;
    return new StaticRange({ startContainer, startOffset, endContainer, endOffset });
  };

  const blankPastPiece = (piece: Element, at: number, back: boolean): Span | null => {
    const own = piece.matches("br") ? null : placeOf(piece, "data-src");
    if (!own) return null;
    const text = hands.text();
    const edge = unitFrom(writer.reckoned, text, back ? own[1] : own[0]);
    if (!/^[ \t]+$/.test(back ? text.slice(edge, at) : text.slice(at, edge))) return null;
    return back ? { from: at - 1, to: at } : { from: at, to: at + 1 };
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

  const headKept = (line: Element) =>
    line.matches(HEADINGS) && (sectioned(page) || line === page.querySelector(TITLE_PLACE));

  const caretFreed = (range: AbstractRange | null) => {
    const at = caret.caretSpan()?.from;
    if (!range?.collapsed || at === undefined) return null;
    const text = hands.text();
    const element = elementOf(range.startContainer);
    const first = (one: Element) => wordsStartAt(one, range.startContainer, range.startOffset);
    const item = element?.closest("li");
    const listed = item && unmarked(text, first(item) ? wordsFrom(text, at) : at);
    if (listed) return listed;
    const line = element?.closest(MARKED_LINE);
    return line && first(line) && !headKept(line) ? unlined(text, wordsFrom(text, at)) : null;
  };

  const keyReach = (
    kind: string,
    text: string,
    reached: Span,
    targeted: boolean,
    spread: boolean,
    held: AbstractRange | null,
  ): KeyReach => {
    const missed = !spread && reached.from === reached.to && (backward(kind) || forward(kind));
    const behind = missed && targeted && backward(kind) ? pictureBeside(kind, reached.from) : null;
    if (behind) return { gone: behind };
    const blind = missed && (forward(kind) || !targeted);
    const unshownKind = kind === "deleteContentForward" || kind === "deleteWordBackward";
    const unshown = !missed && targeted && !spread && unshownKind;
    const at = blind || unshown ? caret.caretSpan()?.from : undefined;
    const passed =
      at === undefined
        ? ""
        : text.slice(...(backward(kind) ? [reached.to, at] : [at, reached.from]));
    const skipped = unshown && /^[ \t]+$/.test(passed);
    const ahead = blind || skipped ? at : undefined;
    const own = ahead === undefined ? null : reachIn(text, ahead, kind);
    const inLine =
      own !== null &&
      own.from >= wordsFrom(text, own.to) &&
      !text.slice(own.from, own.to).includes("\n");
    const reach = own && (inLine || hardBroken(text.slice(own.from, own.to))) ? own : reached;
    const ended = blind && forward(kind) && ahead !== undefined ? joinedAfter(ahead, held) : null;
    return { reach, ended };
  };

  const deleted = (kind: string, text: string, one: Span, how: Deleted) => {
    const { joining, spread, literal, mend } = how;
    const span = unsplit(text, one);
    const keyed =
      !joining && !spread && !kind.includes("Line") && (backward(kind) || forward(kind));
    const reachAt = (from: number) => reachIn(text, from, kind);
    const key = keyed ? { back: backward(kind), reach: reachAt } : null;
    const gone = literal
      ? { ...span, insert: "" }
      : mend
        ? mendedOver(text, span, "", mend).edit
        : keptGone(text, span, key);
    // A title's mark alone is what is left of a page emptied again, so the page is left empty.
    if (titleAloneLeft(text, applied(text, gone))) return emptied(text);
    return put(gone, spread ? gone.from : caretKept(kind, gone.from));
  };

  const keyedDelete = (kind: string, standing: Standing) => {
    const back = backward(kind);
    const { range } = standing;
    const at = caret.caretSpan()?.from;
    if (!range || at === undefined) return false;
    if (
      !range.collapsed &&
      !leftOver(writer, range) &&
      (pastWhole(range) || wholeReach(range) !== null)
    ) {
      retaken(kind, undefined, standing);
      return true;
    }
    const amongWords = range.startContainer.nodeType === Node.TEXT_NODE;
    // WebKit sends no input for a key with nothing it can write past, as a page's last piece, so the key takes it here.
    const last = amongWords && !back ? pastBreak(kind, range) : null;
    if (last) {
      writer.leftFrom = undefined;
      writer.typing = true;
      writer.pending = null;
      tookBeside(kind, last);
      return true;
    }
    if (amongWords && (!back || hands.text().lastIndexOf("\n", at - 1) >= 0)) return false;
    const ahead =
      !back && range.collapsed ? letterAhead(range.startContainer, range.startOffset) : null;
    const [aheadAt, aheadOffset] = ahead ?? [range.startContainer, range.startOffset];
    const spot = new StaticRange({
      startContainer: aheadAt,
      startOffset: aheadOffset,
      endContainer: aheadAt,
      endOffset: aheadOffset,
    });
    const beside = besideCaret(ahead ? spot : range, back);
    if (ahead && !beside?.matches(PIECES)) {
      const joining = nestedJoined(spot, beside);
      if (joining) retaken(kind, joining, standing);
      return true;
    }
    const piece = beside?.matches(PIECES) ? beside : null;
    const gone =
      (piece && blankPastPiece(piece, at, back)) ??
      besideGone(kind, range) ??
      (piece && pieceGone(piece));
    const held = !gone && edgeHeld(kind, range);
    const boxed = !back && Boolean(piece?.matches(".check"));
    const freed = gone || held || (!back && !boxed) ? null : caretFreed(range);
    const acted = gone || freed || held;
    if (!acted && (!piece || wordsBeside(range, !back))) return false;
    if (!gone && !freed) return true;
    writer.leftFrom = undefined;
    writer.typing = true;
    writer.pending = null;
    if (freed) laying.put(freed.edit, freed.caret);
    else if (gone) tookBeside(kind, gone);
    return true;
  };

  return {
    besideGone,
    edgeHeld,
    tookBeside,
    caretFreed,
    unplaced,
    keyReach,
    deleted,
    keyedDelete,
  };
};
