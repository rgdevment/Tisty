import type { Slashed } from "tisty-editor/edit";
import { begunOn } from "./pageBegun";
import { besideOn } from "./pageBeside";
import { caretOn } from "./pageCaret";
import type { Edit } from "./pageEdits";
import { formatsOn } from "./pageFormats";
import { historyOn } from "./pageHistory";
import { inputOn } from "./pageInput";
import { keysOn } from "./pageKeys";
import { layingOn } from "./pageLaying";
import type { Act } from "./pageMoving";
import { inCode, opened } from "./pageNodes";
import { tagList, tagMatch } from "./pageTags";
import { type Hands, writerOf } from "./pageWriter";
import { unformattable } from "./quickly";

export type { Hands, Turns } from "./pageWriter";
export { turnsAnew } from "./pageWriter";

export type PageWriting = {
  readonly drawn: () => void;
  readonly stop: () => void;
  readonly format: (open: string, close: string) => void;
  readonly formattable: () => boolean;
  readonly caretAt: () => number | null;
  readonly placeAt: (at: number, stays?: number) => void;
  readonly rewrite: (next: string) => void;
  readonly outside: (was: string, next: string) => void;
  readonly chosen: () => string | null;
  readonly cut: () => void;
  readonly slashed: (one: Slashed, drawnWhole?: boolean) => Edit | null;
  readonly typed: (said: string) => void;
  readonly unitActed: (top: Element, act: Act) => void;
  readonly begin: (at: number, seed: string, near: Element) => void;
  readonly hintWords: (at: [number, number], hint: HTMLElement) => void;
};

const SETTLED_MS = 300;

export const writtenOn = (page: Document, hands: Hands): PageWriting => {
  const writer = writerOf(page, hands);
  const caret = caretOn(writer);
  const laying = layingOn(writer, caret, () => drawn());
  const history = historyOn(writer, caret, laying);
  const formats = formatsOn(writer, caret, laying);
  const begun = begunOn(writer, caret, laying);

  // Colours, icons and cards land after a block is drawn and take the place of its words, the caret with them.
  const Watching = page.defaultView?.MutationObserver;
  const watching =
    Watching &&
    new Watching((records) => {
      if (records.some((one) => one.type === "attributes")) opened(page);
      if (writer.kept && writer.kept.offset >= 0 && !writer.kept.node.isConnected)
        caret.caretLaid();
    });
  watching?.observe(page.body, { childList: true, subtree: true });
  watching?.observe(page.documentElement, { attributes: true, attributeFilter: ["class"] });

  const beside = besideOn(writer, caret, laying, begun, formats);

  const tags = tagList(page, (match, tag) => {
    writer.typing = false;
    const text = hands.text();
    const to = Math.min(match.to, text.length);
    // A space after the tag lets the next letter start a word, not lengthen the tag.
    const spaced = text[to] === " " ? "" : " ";
    laying.put({ from: Math.min(match.from, to), to, insert: `#${tag}${spaced}` });
  });

  const offerTags = () => {
    const at = writer.kept?.at;
    const range = caret.chosen();
    const known = hands.tags;
    const open =
      writer.typing && at !== undefined && known && !(range && inCode(range.startContainer));
    tags.offer(open && known ? tagMatch(hands.text(), at, known()) : null);
  };

  const drawn = () => {
    writer.whole = false;
    opened(page);
    caret.caretLaid();
    offerTags();
    beside.rest();
    beside.besideLine();
  };

  const input = inputOn(writer, caret, laying, history, formats, begun);
  const keys = keysOn(writer, caret, laying, history, formats, beside, tags);

  page.addEventListener("beforeinput", input.asked);
  page.addEventListener("compositionstart", input.started);
  page.addEventListener("compositionend", input.ended);
  page.addEventListener("mousedown", keys.moved);
  page.addEventListener("click", begun.quoted);
  page.addEventListener("mousedown", keys.ticking);
  page.addEventListener("keydown", keys.moved);
  page.addEventListener("selectionchange", keys.noted);
  page.addEventListener("keydown", keys.keyed);
  page.addEventListener("paste", input.pastedFiles);
  const away = () => hands.settle?.();
  page.defaultView?.addEventListener("blur", away);
  opened(page);
  return {
    drawn,
    format: formats.format,
    formattable: () => !unformattable(page),
    caretAt: () => caret.caretSpan()?.from ?? null,
    placeAt: (at: number, stays?: number) => {
      caret.placeAt(at, stays);
      // Drawings take their height after the first paint, so the + is laid again once they have.
      setTimeout(beside.besideLine, SETTLED_MS);
    },
    rewrite: history.rewrite,
    outside: history.outside,
    chosen: () => {
      const one = caret.chosenNow();
      return one ? hands.text().slice(one.from, one.to) : null;
    },
    slashed: beside.slashed,
    unitActed: beside.unitActed,
    begin: begun.begin,
    hintWords: begun.hintWords,
    typed: begun.typed,
    cut: () => {
      const one = caret.chosenNow();
      if (one) laying.put({ ...one, insert: "" }, one.from);
    },
    stop: () => {
      writer.turns.left = hands.text();
      laying.told();
      hands.settle?.();
      page.defaultView?.removeEventListener("blur", away);
      watching?.disconnect();
      page.removeEventListener("selectionchange", keys.noted);
      page.removeEventListener("keydown", keys.keyed);
      page.removeEventListener("paste", input.pastedFiles);
      tags.stop();
      beside.stop();
      page.removeEventListener("mousedown", keys.moved);
      page.removeEventListener("click", begun.quoted);
      page.removeEventListener("mousedown", keys.ticking);
      page.removeEventListener("keydown", keys.moved);
      page.removeEventListener("beforeinput", input.asked);
      page.removeEventListener("compositionstart", input.started);
      page.removeEventListener("compositionend", input.ended);
      page.querySelector<HTMLElement>(".page")?.removeAttribute("contenteditable");
    },
  };
};
