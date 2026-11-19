import { byteFrom, unitFrom } from "../placing";
import type { Span } from "./pageEdits";
import { closing, opening } from "./pageLists";
import { elementOf, ghostly, lineOfCaret, scrolledAround } from "./pageNodes";
import { placeOf, placeSaid } from "./pagePlaces";
import { endOf, GHOST, ghostAt, sourceAt, spotOf, topsAt } from "./pageSpots";
import type { Point, Writer } from "./pageWriter";

export type Caret = ReturnType<typeof caretOn>;

const WHOLE = '.page > [data-at][contenteditable="false"]:not(h1, h2, h3, h4, h5, h6, .front)';

export const caretOn = (writer: Writer) => {
  const { page, hands } = writer;

  const chosen = (): AbstractRange | null => {
    const held = page.getSelection();
    return held?.rangeCount ? held.getRangeAt(0) : null;
  };

  const spanOfRange = (range: AbstractRange, reached = false): Span | null => {
    const text = hands.text();
    const same = writer.kept && range.collapsed && range.startContainer === writer.kept.node;
    if (same && writer.kept && range.startOffset === writer.kept.offset)
      return { from: writer.kept.at, to: writer.kept.at };
    const known = writer.reckoned;
    const opening = reached && !range.collapsed;
    const from = sourceAt(text, range.startContainer, range.startOffset, known, opening);
    const to = range.collapsed ? from : sourceAt(text, range.endContainer, range.endOffset, known);
    if (from === null || to === null) return null;
    return { from: Math.min(from, to), to: Math.max(from, to) };
  };

  // Out of a paragraph not yet written, going back leaves it and joins the block before.
  const unbegun = (kind: string, range: AbstractRange): Span | null => {
    const unlaid = writer.kept?.lead !== undefined && writer.kept.offset < 0;
    if (!writer.kept || !range.collapsed || !(unlaid || ghostly(range.startContainer))) return null;
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

  const chosenNow = () => {
    const range = chosen();
    const one = range && !range.collapsed ? spanOfRange(range) : writer.lastChosen;
    return one && one.from < one.to ? one : null;
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
    const span = placeOf(box);
    if (!summary || !span) return null;
    const text = hands.text();
    const start = unitFrom(writer.reckoned, text, span[0]);
    const ends = text.indexOf("\n", start);
    const line = text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "");
    const name = start + (/^:::\s*details\s?/.exec(line)?.[0].length ?? line.length);
    if (at < name || at > start + line.length) return null;
    const words = summary.firstChild;
    if (words?.nodeType !== Node.TEXT_NODE) return [summary, 0];
    return [words, Math.min(at - name, (words as Text).length)];
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
    const held = page.getSelection();
    held?.collapse(spot[0], spot[1]);
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
    page.getSelection()?.collapse(spot[0], spot[1]);
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

  // A piece drawn whole, as code or a tag, takes no keys, so a caret dropped inside it stands right after it.
  const outOfSolid = (range: AbstractRange) => {
    const node = range.startContainer;
    const atom = elementOf(node)?.closest('.page [data-src][contenteditable="false"]');
    if (!atom?.parentElement || atom.parentElement.closest('[contenteditable="false"]')) return;
    const early = node.nodeType === Node.TEXT_NODE && range.startOffset * 2 < (node as Text).length;
    const at = Array.prototype.indexOf.call(atom.parentElement.childNodes, atom);
    const [holder, offset] = early ? [atom.parentElement, at] : endOf(atom);
    page.getSelection()?.collapse(holder, offset);
  };

  return {
    chosen,
    spanOfRange,
    unbegun,
    caretSpan,
    chosenNow,
    focused,
    caretLaid,
    placeAt,
    laidAt,
    caretPoint,
    blockEnd,
    paragraphStart,
    blockOfCaret,
    blockStart,
    outOfSolid,
    inSight,
  };
};
