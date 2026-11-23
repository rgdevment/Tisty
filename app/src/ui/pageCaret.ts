import { byteFrom, type Reckoned, unitFrom } from "../placing";
import { blankAt, lineEnd, type Span } from "./pageEdits";
import { foldReached } from "./pageFolds";
import { closing, opening } from "./pageLists";
import { balanced, wordsFrom } from "./pageMarks";
import { carriedOf, mended } from "./pageMends";
import {
  atGhost,
  caretLaidOn,
  elementOf,
  lineOfCaret,
  SOLID_PIECE,
  scrolledAround,
  shownWords,
  unseenLetter,
  WHOLE,
  wholeBehind,
} from "./pageNodes";
import { placeOf, placeSaid } from "./pagePlaces";
import {
  endIn,
  endOf,
  GHOST,
  ghostAt,
  indexOf,
  sourceAt,
  spotOf,
  startOf,
  topsAt,
} from "./pageSpots";
import { type Choice, mendOf, spanReached, wholeReach } from "./pageWholes";
import { choiceKept, keptUnlaid, type Point, type Writer } from "./pageWriter";

export type Caret = ReturnType<typeof caretOn>;

export type Taken = ReturnType<Caret["taken"]>;

type Spot = [Node, number];

export type Standing = { readonly range: AbstractRange | null; readonly lagging: boolean };

const wholeEnd = (text: string, range: AbstractRange, known: Reckoned | null) => {
  const past = placeOf(wholeBehind(range.endContainer, range.endOffset));
  return past ? unitFrom(known, text, past[1]) : null;
};

export const caretOn = (writer: Writer) => {
  const { page, hands } = writer;

  const chosen = (): AbstractRange | null => {
    const held = page.getSelection();
    return held?.rangeCount ? held.getRangeAt(0) : null;
  };

  const nameLine = (text: string, box: Element | null | undefined) => {
    const span = placeOf(box);
    if (!span) return null;
    const start = unitFrom(writer.reckoned, text, span[0]);
    const ends = text.indexOf("\n", start);
    const line = text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "");
    const named = /^[ \t]*(?:>[ \t]?)*[ \t]*:::[ \t]*details[ \t]*/.exec(line);
    const name = start + (named?.[0].length ?? line.length);
    return { name, end: start + line.length, colon: named !== null };
  };

  const nameSource = (text: string, node: Node, offset: number) => {
    const summary = elementOf(node)?.closest(".page details[data-at] > summary");
    const line = summary && nameLine(text, summary.parentElement);
    if (!summary || !line?.colon) return null;
    const before = page.createRange();
    before.selectNodeContents(summary);
    before.setEnd(node, offset);
    return Math.min(line.name + before.toString().length, line.end);
  };

  const spanOfRange = (range: AbstractRange, reached = false): Span | null => {
    const text = hands.text();
    const same = writer.kept && range.collapsed && range.startContainer === writer.kept.node;
    if (same && writer.kept && range.startOffset === writer.kept.offset)
      return { from: writer.kept.at, to: writer.kept.at };
    const known = writer.reckoned;
    const opening = reached && !range.collapsed;
    const pointAt = (node: Node, offset: number, opens: boolean) =>
      nameSource(text, node, offset) ?? sourceAt(text, node, offset, known, opens);
    const from = pointAt(range.startContainer, range.startOffset, opening);
    const to = range.collapsed
      ? from
      : (wholeEnd(text, range, known) ?? pointAt(range.endContainer, range.endOffset, false));
    if (from === null || to === null) return null;
    return { from: Math.min(from, to), to: Math.max(from, to) };
  };

  const shownAt = (at: number): AbstractRange | null => {
    const text = hands.text();
    if (blankAt(text, at)) return null;
    const known = writer.reckoned;
    const spot = spotOf(page, text, at, known);
    if (!spot) return null;
    const [node, offset] = spot;
    const block = placeOf(elementOf(node)?.closest("[data-at]"));
    const lineShown =
      block !== null &&
      unitFrom(known, text, block[0]) <= wordsFrom(text, lineEnd(text, at)) &&
      sourceAt(text, node, offset, known) === at;
    if (!lineShown) return null;
    const Static = page.defaultView?.StaticRange ?? StaticRange;
    return new Static({
      startContainer: node,
      startOffset: offset,
      endContainer: node,
      endOffset: offset,
    });
  };

  const standing = (): Standing => {
    const picked = chosen();
    const kept = writer.kept;
    if (!keptUnlaid(kept) || (picked && !picked.collapsed))
      return { range: picked, lagging: false };
    const text = hands.text();
    if (blankAt(text, kept.at)) return { range: null, lagging: true };
    if (picked && spanOfRange(picked)?.from === kept.at) return { range: picked, lagging: false };
    return { range: shownAt(kept.at), lagging: true };
  };

  // Out of a paragraph not yet written, going back leaves it and joins the block before.
  const unbegun = (kind: string, range: AbstractRange): Span | null => {
    const unlaid = writer.kept?.lead !== undefined && writer.kept.offset < 0;
    const begun = unlaid || atGhost(range.startContainer, range.startOffset);
    if (!writer.kept || !range.collapsed || !begun) return null;
    const { at } = writer.kept;
    if (!kind.startsWith("delete") || kind.includes("Forward")) return { from: at, to: at };
    const text = hands.text();
    let from = at;
    while (from > 0 && /\s/.test(text[from - 1] ?? "")) from--;
    return { from, to: at };
  };

  const caretSpan = () => {
    if (writer.kept && writer.kept.offset < 0) return { from: writer.kept.at, to: writer.kept.at };
    const range = chosen();
    return range && (unbegun("insertText", range) ?? spanOfRange(range));
  };

  const spanChosen = (range: AbstractRange): Span | null => {
    const span = spanReached(range, spanOfRange(range), hands.text(), writer.reckoned);
    const reach = span && foldReached(range);
    if (!span || !reach) return span;
    const text = hands.text();
    const place = reach.whole ? placeOf(reach.block) : null;
    let end = place
      ? unitFrom(writer.reckoned, text, place[1])
      : (sourceAt(text, ...endIn(reach.block), writer.reckoned) ?? span.to);
    while (end > span.from && /[\r\n]/.test(text[end - 1] ?? "")) end--;
    return { from: span.from, to: end };
  };

  const chosenAs = (spanOf: (range: AbstractRange) => Span | null) => {
    const kept = choiceKept(writer.kept);
    if (kept) return kept;
    const range = chosen();
    const one = range && !range.collapsed ? spanOf(range) : writer.lastChosen;
    return one && one.from < one.to ? one : null;
  };

  const chosenNow = () => chosenAs(spanOfRange);

  const choiceOf = (range: AbstractRange): Choice | null => {
    const text = hands.text();
    const reach = wholeReach(range);
    const span = spanReached(range, spanOfRange(range), text, writer.reckoned, reach);
    return span && { span, mend: mendOf(range, text, writer.reckoned, reach) };
  };

  const takenFrom = (): Choice | null => {
    const kept = choiceKept(writer.kept);
    if (kept) return { span: kept, mend: null };
    const range = chosen();
    const last = writer.lastRange;
    if (range && !range.collapsed) return choiceOf(range);
    return last && !last.collapsed ? choiceOf(last) : null;
  };

  const taken = () => {
    const choice = takenFrom();
    if (!choice || choice.span.from >= choice.span.to) return null;
    const text = hands.text();
    const span = balanced(text, choice.span);
    const { mend } = choice;
    const left = mend ? mended(text, { ...span, insert: "" }, mend).insert : "";
    return { ...span, carried: carriedOf(text, span, mend), left };
  };

  // A menu of the window hands the focus back to the frame, not to the words, so the sheet takes it again.
  const focused = () => {
    const sheet = page.querySelector<HTMLElement>(".page");
    if (!page.hasFocus() || !sheet || sheet.contains(page.activeElement)) return;
    const held = page.getSelection();
    const caret = held?.rangeCount ? held.getRangeAt(0).cloneRange() : null;
    sheet.focus({ preventScroll: true });
    if (!caret || !sheet.contains(caret.startContainer)) return;
    held?.removeAllRanges();
    held?.addRange(caret);
  };

  const ROOM = 48;

  const inSightAround = (box: DOMRect) => {
    const frame = page.defaultView?.frameElement as HTMLElement | null;
    if (!frame) return;
    for (const at of scrolledAround(page)) {
      const area = at.getBoundingClientRect();
      const top = frame.getBoundingClientRect().top + box.top;
      const bottom = top + box.height;
      if (bottom > area.bottom - ROOM) at.scrollBy({ top: bottom - area.bottom + ROOM });
      else if (top < area.top + ROOM / 2) at.scrollBy({ top: top - area.top - ROOM / 2 });
    }
  };

  // A caret laid by a script never scrolls, so the line being written is brought back when it slips out of sight.
  const inSight = (holder: Node) => {
    const view = page.defaultView;
    const held = page.getSelection();
    if (!view || !page.hasFocus() || !held?.rangeCount) return;
    const range = held.getRangeAt(0);
    const line = elementOf(holder);
    let box =
      typeof range.getBoundingClientRect === "function" ? range.getBoundingClientRect() : null;
    if (!box?.height && line) box = line.getBoundingClientRect();
    if (!box) return;
    if (page.documentElement.scrollHeight <= view.innerHeight + 1) return inSightAround(box);
    if (box.bottom > view.innerHeight - ROOM)
      view.scrollBy({ top: box.bottom - view.innerHeight + ROOM });
    else if (box.top < ROOM / 2) view.scrollBy({ top: box.top - ROOM / 2 });
  };

  // A collapsible's name is drawn in its summary with no place marked in the text, so the caret is counted there.
  const summaryAt = (found: [Node, number], at: number): [Node, number] | null => {
    const box = elementOf(found[0])?.closest("details[data-at]");
    const summary = box?.querySelector(":scope > summary");
    const line = nameLine(hands.text(), box);
    if (!summary || !line || at < line.name || at > line.end) return null;
    const words = summary.firstChild;
    if (words?.nodeType !== Node.TEXT_NODE) return [summary, 0];
    return [words, Math.min(at - line.name, (words as Text).length)];
  };

  // A block drawn whole, as a widget, a formula, a table or a break, takes no caret in its source.
  const pastWhole = (text: string, at: number) => {
    const sheet = page.querySelector(".page");
    const byte = byteFrom(writer.reckoned, text, at);
    for (const one of sheet ? topsAt(sheet, byte) : []) {
      const span = placeOf(one);
      if (!span || !one.matches(WHOLE)) continue;
      let end = unitFrom(writer.reckoned, text, span[1]);
      while (end > 0 && /[\r\n]/.test(text[end - 1] ?? "")) end--;
      return end;
    }
    return null;
  };

  const beforeWhole = (text: string, at: number) => {
    const sheet = page.querySelector(".page");
    const byte = byteFrom(writer.reckoned, text, at);
    for (const top of sheet ? topsAt(sheet, byte) : []) {
      const solid = top.querySelectorAll('[data-at][contenteditable="false"]');
      const inner = Array.from(solid).find((one) => {
        const span = placeOf(one);
        return span !== null && span[0] <= byte && byte < span[1] && one.matches(WHOLE);
      });
      if (inner?.parentNode)
        return sourceAt(text, inner.parentNode, indexOf(inner), writer.reckoned);
    }
    return null;
  };

  // The properties are kept off the page, so a caret come back from them stands where the words begin.
  const pastFront = (text: string, at: number) => {
    const first = page.querySelector(".page > .front[data-at]");
    const own = first?.nextElementSibling;
    const front = own?.matches(".front.agent-own[data-at]") ? own : first;
    const span = placeOf(front);
    if (!span || byteFrom(writer.reckoned, text, at) > span[1]) return null;
    const next = placeOf(front?.nextElementSibling);
    if (!next) return null;
    const start = unitFrom(writer.reckoned, text, next[0]);
    return start + (/^[ \t]*(?:#{1,6}[ \t]+)?/.exec(text.slice(start))?.[0].length ?? 0);
  };

  const caretLaid = () => {
    if (!writer.kept) return;
    const text = hands.text();
    const front = writer.kept.lead === undefined ? pastFront(text, writer.kept.at) : null;
    if (front !== null) writer.kept = { node: page.body, offset: -1, at: front };
    const shaping = page.documentElement.classList.contains("shaping");
    const past =
      writer.kept.lead === undefined && !shaping ? pastWhole(text, writer.kept.at) : null;
    if (past !== null)
      writer.kept = {
        node: page.body,
        offset: -1,
        at: past,
        lead: opening(text, past, ""),
        tail: closing(text, past),
      };
    const within =
      writer.kept.lead === undefined && !shaping ? beforeWhole(text, writer.kept.at) : null;
    if (within !== null) writer.kept = { node: page.body, offset: -1, at: within };
    const known = writer.reckoned;
    const spotAt = (at: number) => spotOf(page, hands.text(), at, known);
    const unwritten = writer.kept.lead !== undefined || !spotAt(writer.kept.at);
    const ghost = unwritten
      ? ghostAt(page, hands.text(), writer.kept.at, known, writer.kept.lead)
      : null;
    const found: [Node, number] | null = ghost ? [ghost, 0] : spotAt(writer.kept.at);
    if (!found) return;
    focused();
    const spot = summaryAt(found, writer.kept.at) ?? found;
    // A block drawn empty, as a heading just begun, holds no line for the caret, so it gets one.
    const [holder] = spot;
    const bare = holder.nodeType === Node.ELEMENT_NODE ? (holder as Element) : null;
    if (bare && !bare.textContent && !bare.querySelector("br"))
      bare.append(page.createElement("br"));
    else if (bare?.matches("li") && elementOf(bare.childNodes[spot[1]])?.matches("ul, ol"))
      bare.insertBefore(page.createElement("br"), bare.childNodes[spot[1]] ?? null);
    caretLaidOn(page, spot[0], spot[1]);
    const from = writer.kept.chosenFrom === undefined ? null : spotAt(writer.kept.chosenFrom);
    if (from) page.getSelection()?.setBaseAndExtent(...from, ...spot);
    writer.kept = { ...writer.kept, node: spot[0], offset: spot[1] };
    inSight(holder);
  };

  // `stays` is how far down the window the caret's line was, so a view handed back keeps the line where the eye was.
  const placeAt = (at: number, stays?: number) => {
    writer.kept = { node: page.body, offset: -1, at: Math.min(at, hands.text().length) };
    page.querySelector<HTMLElement>(".page")?.focus({ preventScroll: true });
    caretLaid();
    const view = page.defaultView;
    const range = page.getSelection()?.rangeCount ? page.getSelection()?.getRangeAt(0) : null;
    const box = range?.getBoundingClientRect?.();
    const scrolls = view && page.documentElement.scrollHeight > view.innerHeight + 1;
    if (stays !== undefined && scrolls && box?.height)
      return view.scrollBy({ top: box.top - stays });
    elementOf(writer.kept?.node)?.scrollIntoView?.({ block: "nearest" });
  };

  const laidAt = (spot: [Node, number]) => {
    caretLaidOn(page, spot[0], spot[1]);
    inSight(spot[0]);
  };

  const reachedTo = (spot: [Node, number]) => {
    const held = page.getSelection();
    if (!held?.rangeCount) return laidAt(spot);
    held.extend(spot[0], spot[1]);
    inSight(spot[0]);
  };

  // Where the menu of blocks opens: under the caret, on the window's own screen.
  const caretPoint = (): Point => {
    const held = page.getSelection();
    const range = held?.rangeCount ? held.getRangeAt(0) : null;
    const ghost = page.querySelector(GHOST);
    let box = range?.getBoundingClientRect();
    if (!box || (box.width === 0 && box.height === 0))
      box = (ghost ?? lineOfCaret(range))?.getBoundingClientRect() ?? box;
    const frame = page.defaultView?.frameElement?.getBoundingClientRect();
    return {
      x: (frame?.left ?? 0) + (box?.left ?? 0),
      y: (frame?.top ?? 0) + (box?.bottom ?? 0) + 10,
      top: (frame?.top ?? 0) + (box?.top ?? 0) - 4,
    };
  };

  const blockEnd = (at: number) => {
    const anchor = elementOf(page.getSelection()?.anchorNode);
    const span = placeOf(anchor?.closest(".page > [data-at]"));
    return span ? unitFrom(writer.reckoned, hands.text(), span[1]) : at;
  };

  // Only a paragraph is turned whole, from its first line, as a line broken inside it still belongs to it.
  const paragraphStart = () => {
    const anchor = elementOf(page.getSelection()?.anchorNode);
    const span = placeOf(anchor?.closest(".page > p[data-at]"));
    return span ? unitFrom(writer.reckoned, hands.text(), span[0]) : null;
  };

  const blockOfCaret = () =>
    elementOf(chosen()?.startContainer)?.closest(".page [data-at]") ?? null;

  // A block is told apart by where it starts, since each key draws it again as another element.
  const blockStart = () => placeSaid(blockOfCaret())?.split("-")[0] ?? null;

  const writtenSolid = (node: Node) => {
    const atom = elementOf(node)?.closest(`.page ${SOLID_PIECE}`);
    const free = atom?.parentElement && !atom.parentElement.closest('[contenteditable="false"]');
    return free ? atom : null;
  };

  // A piece drawn whole, as code or a tag, takes no keys, so a caret dropped inside it stands right after it.
  const outOfSolid = (range: AbstractRange) => {
    const node = range.startContainer;
    const atom = writtenSolid(node);
    if (!atom) return;
    const early =
      node.nodeType === Node.TEXT_NODE &&
      !unseenLetter(node) &&
      range.startOffset * 2 < (node as Text).length;
    const [holder, offset] = early ? startOf(atom) : endOf(atom);
    caretLaidOn(page, holder, offset);
  };

  const wordsOf = (atom: Element, node: Node, offset: number, before: boolean) => {
    const part = page.createRange();
    part.selectNodeContents(atom);
    if (before) part.setEnd(node, offset);
    else part.setStart(node, offset);
    return shownWords(part.toString()) !== "";
  };

  const opensIn = (atom: Element, node: Node, offset: number) =>
    wordsOf(atom, node, offset, false) ? startOf(atom) : endOf(atom);

  const endsIn = (atom: Element, node: Node, offset: number) =>
    wordsOf(atom, node, offset, true) ? endOf(atom) : startOf(atom);

  const madeBackward = (held: Selection) => {
    if (!held.anchorNode || !held.focusNode) return false;
    const anchor = page.createRange();
    anchor.setStart(held.anchorNode, held.anchorOffset);
    return anchor.comparePoint(held.focusNode, held.focusOffset) < 0;
  };

  const solidsLeft = () => {
    const held = page.getSelection();
    const range = chosen();
    if (!held || !range) return;
    if (range.collapsed) return outOfSolid(range);
    const { startContainer, startOffset, endContainer, endOffset } = range;
    const first = writtenSolid(startContainer);
    const last = writtenSolid(endContainer);
    if (!first && !last) return;
    const from: Spot = first
      ? opensIn(first, startContainer, startOffset)
      : [startContainer, startOffset];
    const to: Spot = last ? endsIn(last, endContainer, endOffset) : [endContainer, endOffset];
    if (madeBackward(held)) held.setBaseAndExtent(...to, ...from);
    else held.setBaseAndExtent(...from, ...to);
  };

  return {
    chosen,
    spanOfRange,
    shownAt,
    standing,
    unbegun,
    caretSpan,
    spanChosen,
    chosenNow,
    taken,
    focused,
    caretLaid,
    placeAt,
    laidAt,
    reachedTo,
    caretPoint,
    blockEnd,
    paragraphStart,
    blockOfCaret,
    blockStart,
    outOfSolid,
    solidsLeft,
    inSight,
  };
};
