import type { Begun } from "./pageBegun";
import { blank } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { lineEnd, type Span } from "./pageEdits";
import { fenceJustOpened, fenceNamed, fenceOpened } from "./pageFences";
import type { Formats } from "./pageFormats";
import { inOneCodeBlock, leftOver } from "./pageInputEdges";
import { mendedOver, type Putting } from "./pageLaidOver";
import {
  closing,
  hardBreak,
  leftBehind,
  openedAbove,
  opening,
  outdented,
  parted,
} from "./pageLists";
import { closedRun } from "./pageMarks";
import type { Mend } from "./pageMends";
import { atGhost, elementOf, inCode, onSheet, sectioned } from "./pageNodes";
import { mendOf } from "./pageWholes";
import { compositionOpened, type Writer } from "./pageWriter";

type Composing = {
  readonly one: Span;
  readonly range: Range;
  readonly was: string;
  readonly mend: Mend | null;
};

export const typingOn = (
  writer: Writer,
  caret: Caret,
  formats: Formats,
  begun: Begun,
  { put, putOver }: Putting,
) => {
  const { page, hands } = writer;
  let composing: Composing | null = null;

  // A slash opens the blocks where a word could start, as it always did; inside code it is only a slash.
  const slashes = (text: string, one: Span, range: AbstractRange) =>
    one.from === one.to &&
    !inCode(range.startContainer) &&
    (blank(text, one.from) || /(^|\s)$/.test(text.slice(0, one.from).split("\n").pop() ?? ""));

  const brokenAt = (text: string, at: number, to: number, mend: Mend | null) => {
    const lead = hardBreak(text, at);
    const laid = mend && mendedOver(text, { from: at, to }, "", mend);
    if (laid) return put(laid.edit, laid.edit.from + laid.edit.insert.length);
    if (text.slice(to, lineEnd(text, to)).trim()) return put({ from: at, to, insert: lead });
    if (to > at) put({ from: at, to, insert: "" });
    writer.kept = { node: page.body, offset: -1, at, lead, tail: "" };
    caret.caretLaid();
  };

  const typedIn = (one: Span, range: AbstractRange, typed: string, mend: Mend | null = null) => {
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
    putOver(one, typed, inOneCodeBlock(range), mend);
  };

  const lineParted = (
    kind: string,
    text: string,
    one: Span,
    range: AbstractRange,
    coded: boolean,
    mend: Mend | null,
  ) => {
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
    const named =
      kind === "insertParagraph" && coded && one.from === one.to && fenceNamed(text, one.from);
    if (named) return put(named.edit, named.caret);
    if (kind === "insertParagraph" && coded && fenceJustOpened(text, one.from)) return;
    const at = one.from === one.to && !coded ? closedRun(text, one.from) : one.from;
    const to = Math.max(at, one.to);
    if (kind === "insertLineBreak" && !coded) return brokenAt(text, at, to, mend);
    const insert = kind === "insertLineBreak" ? "\n" : parted(text, at, coded);
    if (!mend) return put({ from: at, to, insert });
    const laid = mendedOver(text, { from: at, to }, insert, mend);
    return put(laid.edit, mend.last ? laid.edit.from + laid.edit.insert.length : laid.caret);
  };

  const started = (event: CompositionEvent) => {
    if (!onSheet(page, event)) return;
    writer.composing ??= compositionOpened();
    const held = page.getSelection();
    const range = held?.rangeCount ? held.getRangeAt(0).cloneRange() : null;
    const spread = Boolean(range && !range.collapsed && !leftOver(writer, range));
    const one = range && ((!spread && begun.waited("insertText", null)) || caret.spanChosen(range));
    const mend = range && spread ? mendOf(range, hands.text(), writer.reckoned) : null;
    composing = range && one ? { one, range, was: hands.text(), mend } : null;
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
      if (same && held.one.from < held.one.to)
        putOver(held.one, "", inOneCodeBlock(held.range), held.mend);
      return;
    }
    const kept = writer.kept;
    const { startContainer, startOffset } = held.range;
    if (kept?.lead !== undefined && (kept.offset < 0 || atGhost(startContainer, startOffset)))
      return typed === "/" ? hands.blocks(caret.caretPoint(), true) : begun.firstKey(kept, typed);
    typedIn(held.one, held.range, typed, held.mend);
  };

  return { typedIn, lineParted, started, ended };
};
