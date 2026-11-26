import { bytesIn, unitFrom } from "../placing";
import { onMac } from "./macKeys";
import type { Caret } from "./pageCaret";
import { applied, type Edit } from "./pageEdits";
import type { Laying } from "./pageLaying";
import { keyWritten, placeOf } from "./pagePlaces";
import { recorded, type Writer } from "./pageWriter";

const SWITCH = "tisty.nativeTyping";

/** The browser writes plain words itself only where it was tried: on Windows and Linux once switched on, on a Mac only when asked by name. */
export const nativeTyping = (): boolean => {
  const said = globalThis.localStorage?.getItem(SWITCH) ?? null;
  return said === "everywhere" || (said === "on" && !onMac);
};

const KINDS = new Set([
  "insertText",
  "insertReplacementText",
  "deleteContentBackward",
  "deleteContentForward",
  "deleteWordBackward",
  "deleteWordForward",
]);

// Any letter the language reads as a mark, a line's opening or a menu's call goes the slow way, which knows them.
const MARKED = /[\\*_~=+`[\]#>|!$:<&{}/@\-\r\n\t\u200B]/;
const NUMBERED = /^\d{1,9}[.)]$/;
const AROUND = new Set(["STRONG", "EM", "S", "DEL", "U", "MARK", "B", "I"]);
const LINES =
  "p[data-at], h1[data-at], h2[data-at], h3[data-at], h4[data-at], h5[data-at], h6[data-at], li[data-at]";

const evened = (said: string) => said.replace(/\u00A0/g, " ");

type Held = {
  readonly kind: string;
  readonly node: Text;
  readonly span: Element;
  readonly top: Element;
  readonly before: string;
  readonly start: number;
  readonly from: number;
  readonly stood: number;
  readonly meant: string;
  readonly landing: number;
  readonly reach: readonly [number, number];
  readonly typed: string;
  readonly text: string;
};

// Only a span drawn for plain words, among nothing but emphasis inside a line, is written in by the browser.
const plainSpan = (node: Node): { span: Element; top: Element } | null => {
  const span = node.parentElement;
  if (span?.tagName !== "SPAN" || span.attributes.length !== 1) return null;
  if (!placeOf(span, "data-src") || span.childNodes.length !== 1) return null;
  let at = span.parentElement;
  while (at && !at.matches("[data-at]")) {
    if (!AROUND.has(at.tagName)) return null;
    at = at.parentElement;
  }
  if (!at?.matches(LINES)) return null;
  let top: Element = at;
  while (top.parentElement && !top.parentElement.matches(".page")) top = top.parentElement;
  return top.parentElement ? { span, top } : null;
};

export type Native = ReturnType<typeof nativeOn>;

export const nativeOn = (writer: Writer, caret: Caret, laying: Laying, typedAfter: () => void) => {
  const { page, hands } = writer;
  const on = nativeTyping();
  let held: Held | null = null;

  // The key's reach and what is chosen both lie in one run of words, or the key reaches past what the browser can write.
  const aimedAt = (event: InputEvent): AbstractRange | null => {
    const chosen = caret.chosen();
    if (!chosen || chosen.startContainer !== chosen.endContainer) return null;
    const [named] = event.getTargetRanges?.() ?? [];
    if (!named) return chosen;
    const within =
      named.startContainer === chosen.startContainer &&
      named.endContainer === chosen.startContainer;
    return within ? named : null;
  };

  // A spot the editor keeps for this very point of the page, other than the one it shows, is the slow way's to keep.
  const keptElsewhere = (node: Node, offset: number, at: number) => {
    const kept = writer.kept;
    return kept !== null && kept.node === node && kept.offset === offset && kept.at !== at;
  };

  const free = () =>
    on &&
    laying.idle() &&
    !writer.whole &&
    !writer.composing &&
    writer.pending === null &&
    writer.carried === null &&
    writer.kept?.lead === undefined &&
    !page.documentElement.classList.contains("untitled");

  const allowed = (kind: string, data: string, node: Text, from: number, to: number) => {
    if (!KINDS.has(kind)) return false;
    if (kind.startsWith("delete")) return to > from && to - from < node.length;
    if (!data || MARKED.test(data)) return false;
    const spaced = /\s/.test(data);
    if (!spaced) return true;
    // A space beside emphasis would end or open it, and after a number at a line's start it opens a list.
    const around = node.parentElement?.parentElement;
    if (around && AROUND.has(around.tagName) && (from === 0 || to === node.length)) return false;
    return !(from === 0 && to === 0) && !NUMBERED.test(node.data.slice(0, from).trimStart());
  };

  const taken = (event: InputEvent): boolean => {
    held = null;
    if (!free() || event.isComposing) return false;
    const kind = event.inputType;
    const data =
      kind === "insertReplacementText"
        ? (event.dataTransfer?.getData("text/plain") ?? event.data ?? "")
        : (event.data ?? "");
    const aim = aimedAt(event);
    if (!aim || aim.startContainer !== aim.endContainer) return false;
    const node = aim.startContainer;
    if (node.nodeType !== Node.TEXT_NODE) return false;
    const text = node as Text;
    const placed = plainSpan(text);
    if (!placed || !allowed(kind, data, text, aim.startOffset, aim.endOffset)) return false;
    const now = hands.text();
    const bytes = placeOf(placed.span, "data-src");
    if (!bytes) return false;
    const start = unitFrom(writer.reckoned, now, bytes[0]);
    const end = unitFrom(writer.reckoned, now, bytes[1]);
    // The words shown must be the very words written, or the browser's change could not be read back.
    if (evened(now.slice(start, end)) !== evened(text.data)) return false;
    const stood = start + (kind.endsWith("Backward") ? aim.endOffset : aim.startOffset);
    const chosen = caret.chosen();
    if (chosen && keptElsewhere(text, chosen.startOffset, start + chosen.startOffset)) return false;
    const insert = kind.startsWith("delete") ? "" : data;
    const meant = text.data.slice(0, aim.startOffset) + insert + text.data.slice(aim.endOffset);
    const landing = aim.startOffset + insert.length;
    held = {
      kind,
      node: text,
      ...placed,
      before: text.data,
      start,
      from: bytes[0],
      stood,
      meant,
      landing,
      reach: [aim.startOffset, aim.endOffset],
      typed: insert,
      text: now,
    };
    writer.leftFrom = undefined;
    writer.typing = true;
    return true;
  };

  const caretAfter = (one: Held, edit: Edit) => {
    const chosen = caret.chosen();
    const inside = chosen?.collapsed && chosen.startContainer === one.node;
    return inside ? one.start + chosen.startOffset : edit.from + edit.insert.length;
  };

  // Whatever the browser did that cannot be read back as words in their span is drawn again from what is written.
  const redrawn = (at: number) => {
    writer.kept = { node: page.body, offset: -1, at };
    writer.reckoned = null;
    writer.whole = true;
    hands.redraw(hands.text());
  };

  // Only the input of the key taken is read back; any other, as a composition's, is not this key's.
  const written = (event: Event) => {
    const one = held;
    held = null;
    if (!one || (event as InputEvent).inputType !== one.kind || (event as InputEvent).isComposing)
      return;
    const { node, span } = one;
    if (hands.text() !== one.text) return redrawn(one.stood);
    const kept = node.isConnected && span.childNodes.length === 1 && span.firstChild === node;
    if (!kept || !node.data) return redrawn(one.stood);
    // The browser may tidy the blanks around a key, as two spaces shown as one; the page keeps what the key meant.
    if (evened(node.data) !== evened(one.meant)) {
      node.data = one.meant;
      page.getSelection()?.collapse(node, Math.min(one.landing, node.length));
    }
    // The page now shows what the key meant, so the words change where it aimed, never guessed among blanks around it.
    const change = { from: one.reach[0], to: one.reach[1], insert: one.typed };
    if (change.from === change.to && !change.insert) return;
    const edit = {
      from: one.start + change.from,
      to: one.start + change.to,
      insert: change.insert,
    };
    const next = applied(one.text, edit);
    const after = caretAfter(one, edit);
    recorded(writer.turns, {
      from: edit.from,
      removed: one.text.slice(edit.from, edit.to),
      added: edit.insert,
      at: one.stood,
      after,
      typed: Date.now(),
    });
    const from = one.from + bytesIn(one.text, one.start, edit.from);
    const removed = bytesIn(one.text, edit.from, edit.to);
    const added = bytesIn(edit.insert);
    keyWritten(one.top, span, from + removed, added - removed);
    writer.kept = { node, offset: after - one.start, at: after };
    writer.leftChosen = null;
    hands.write(next);
    laying.typedNatively(one.text, next, edit, from);
    typedAfter();
  };

  return { taken, written, on };
};
