import type { Slashed } from "tisty-editor/edit";
import { begunOn } from "./pageBegun";
import { besideOn } from "./pageBeside";
import { caretOn } from "./pageCaret";
import { choosingOn } from "./pageChoosing";
import { draggingOn } from "./pageDragging";
import type { Edit } from "./pageEdits";
import { formatsOn } from "./pageFormats";
import { historyOn } from "./pageHistory";
import { inputOn } from "./pageInput";
import { keysOn } from "./pageKeys";
import { layingOn } from "./pageLaying";
import { balanced } from "./pageMarks";
import type { Act } from "./pageMoving";
import { elementOf, inCode, onSheet, opened } from "./pageNodes";
import { linkList, linkMatch, tagList, tagMatch } from "./pageTags";
import { composingKey, type Hands, writerOf } from "./pageWriter";
import { unformattable } from "./quickly";

export type { Hands, Turns } from "./pageWriter";
export { recorded, turnOf, turnsAnew } from "./pageWriter";

export type PageWriting = {
  readonly drawn: () => void;
  readonly stop: () => boolean;
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
  readonly held: () => () => void;
  readonly unitActed: (top: Element, act: Act) => void;
  readonly begin: (at: number, seed: string, near: Element) => void;
  readonly hintWords: (at: [number, number], hint: HTMLElement) => void;
  readonly heldAcross: (was: string, next: string) => void;
};

const SETTLED_MS = 300;

const EDGE_FIRST = ["keydown", "beforeinput", "compositionstart"] as const;

export const writtenOn = (page: Document, hands: Hands): PageWriting => {
  const writer = writerOf(page, hands);
  const caret = caretOn(writer);
  const laying = layingOn(writer, caret, (laid) => drawn(laid));
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

  const links = linkList(page, (match, one) => {
    writer.typing = false;
    const to = Math.min(match.to, hands.text().length);
    laying.put({ from: Math.min(match.from, to), to, insert: one.link });
  });

  const offerLists = () => {
    const at = writer.kept?.at;
    const range = caret.chosen();
    const known = hands.tags;
    const open = writer.typing && at !== undefined && !(range && inCode(range.startContainer));
    const linking = open ? linkMatch(hands.text(), at, hands, offerLists) : null;
    links.offer(linking);
    tags.offer(open && known && !linking ? tagMatch(hands.text(), at, known()) : null);
  };
  const lists = {
    hide: () => {
      tags.hide();
      links.hide();
    },
    stop: () => {
      tags.stop();
      links.stop();
    },
  };

  const caretInSight = () => {
    const at = page.getSelection()?.anchorNode;
    if (at) caret.inSight(at);
  };

  const drawn = (laid?: readonly Element[]) => {
    writer.whole = false;
    if (!laid) opened(page);
    laying.landed(caret.caretLaid, caretInSight);
    offerLists();
    beside.rest();
    beside.besideLine();
    choosing.shown();
  };

  const input = inputOn(writer, caret, laying, history, formats, begun);
  const choosing = choosingOn(writer, caret, laying, beside);
  const keys = keysOn(writer, caret, laying, history, formats, beside, lists, choosing);
  const chooser = (event: KeyboardEvent) => {
    if (onSheet(page, event) && choosing.keyed(event)) event.stopImmediatePropagation();
  };
  const unchosen = () => choosing.letGo(false);
  const dragging = draggingOn(writer, caret, laying);

  for (const first of EDGE_FIRST) page.addEventListener(first, keys.edgeLaid, true);
  page.addEventListener("mousedown", keys.edgeClicked, true);
  page.addEventListener("beforeinput", input.asked);
  page.addEventListener("compositionstart", input.started);
  page.addEventListener("compositionend", input.ended);
  page.addEventListener("mousedown", keys.moved);
  page.addEventListener("click", begun.quoted);
  page.addEventListener("mousedown", keys.ticking);
  page.addEventListener("keydown", keys.moved);
  page.addEventListener("selectionchange", keys.noted);
  // Enter on a collapsible's name would fold it, as a click does, so it opens a line under the name instead.
  const named = (event: KeyboardEvent) => {
    if (event.key !== "Enter" || event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.shiftKey && composingKey(event)) return;
    const fold = elementOf(caret.chosen()?.startContainer)?.closest(
      ".page details[data-at] > summary",
    )?.parentElement as HTMLDetailsElement | null | undefined;
    if (!fold) return;
    event.preventDefault();
    begun.intoFold(fold);
  };
  const chosen = () => {
    const text = hands.text();
    const blocks = choosing.span();
    if (blocks) return text.slice(blocks.from, blocks.to);
    const one = caret.chosenNow();
    const whole = one && balanced(text, one);
    return whole ? text.slice(whole.from, whole.to) : null;
  };
  const cut = () => {
    if (choosing.choosing()) return choosing.dropped();
    const picked = caret.chosenNow();
    const one = picked && balanced(hands.text(), picked);
    if (one) laying.put({ ...one, insert: "" }, one.from);
  };
  // Ctrl+C and Ctrl+X carry the Tisty Markdown of the words chosen, as the right-click does, never the drawn page.
  const clipped = (event: ClipboardEvent) => {
    const words = onSheet(page, event) ? chosen() : null;
    if (!words || !event.clipboardData) return;
    event.preventDefault();
    event.clipboardData.setData("text/plain", words);
    if (event.type === "cut") cut();
  };
  page.addEventListener("keydown", named, true);
  page.addEventListener("keydown", keys.outOfSolids, true);
  page.addEventListener("keydown", chooser);
  page.addEventListener("keydown", keys.keyed);
  page.addEventListener("keydown", input.markHeld);
  page.addEventListener("keydown", input.lineBroken);
  page.addEventListener("mousedown", unchosen);
  page.addEventListener("paste", input.pastedFiles);
  page.addEventListener("copy", clipped);
  page.addEventListener("cut", clipped);
  const away = () => hands.settle?.();
  page.defaultView?.addEventListener("blur", away);
  opened(page);
  let settling: ReturnType<typeof setTimeout> | undefined;
  return {
    drawn,
    format: formats.format,
    formattable: () => !unformattable(page),
    caretAt: () => caret.caretSpan()?.from ?? null,
    placeAt: (at: number, stays?: number) => {
      caret.placeAt(at, stays);
      // Drawings take their height after the first paint, so the + is laid again once they have.
      clearTimeout(settling);
      settling = setTimeout(beside.besideLine, SETTLED_MS);
    },
    rewrite: history.rewrite,
    outside: history.outside,
    chosen,
    slashed: beside.slashed,
    unitActed: beside.unitActed,
    begin: begun.begin,
    hintWords: begun.hintWords,
    heldAcross: laying.heldAcross,
    typed: begun.typed,
    // Where the caret stood when a menu opened, so words it hands back land there whatever was clicked since.
    held: () => {
      const kept = writer.kept ? { ...writer.kept } : null;
      const at = caret.caretSpan()?.from ?? null;
      return () => {
        if (kept?.lead !== undefined) {
          writer.kept = { ...kept, node: page.body, offset: -1 };
          return caret.caretLaid();
        }
        if (at !== null) caret.placeAt(at);
      };
    },
    cut,
    stop: () => {
      clearTimeout(settling);
      writer.turns.left = hands.text();
      writer.composing?.end();
      writer.composing = null;
      const unlaid = laying.stop();
      hands.settle?.();
      page.defaultView?.removeEventListener("blur", away);
      watching?.disconnect();
      page.removeEventListener("selectionchange", keys.noted);
      page.removeEventListener("keydown", keys.keyed);
      page.removeEventListener("keydown", input.markHeld);
      page.removeEventListener("keydown", input.lineBroken);
      page.removeEventListener("keydown", chooser);
      page.removeEventListener("mousedown", unchosen);
      page.removeEventListener("keydown", named, true);
      page.removeEventListener("keydown", keys.outOfSolids, true);
      page.removeEventListener("paste", input.pastedFiles);
      page.removeEventListener("copy", clipped);
      page.removeEventListener("cut", clipped);
      lists.stop();
      dragging.stop();
      beside.stop();
      page.removeEventListener("mousedown", keys.moved);
      page.removeEventListener("click", begun.quoted);
      page.removeEventListener("mousedown", keys.ticking);
      page.removeEventListener("keydown", keys.moved);
      page.removeEventListener("beforeinput", input.asked);
      for (const first of EDGE_FIRST) page.removeEventListener(first, keys.edgeLaid, true);
      page.removeEventListener("mousedown", keys.edgeClicked, true);
      keys.edgeLetGo();
      page.removeEventListener("compositionstart", input.started);
      page.removeEventListener("compositionend", input.ended);
      page.querySelector<HTMLElement>(".page")?.removeAttribute("contenteditable");
      return unlaid;
    },
  };
};
