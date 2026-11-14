import { charsAt, spanned } from "tisty-editor/read";
import type { Begun } from "./pageBegun";
import { blank, blockPlaced } from "./pageBlocks";
import type { Caret } from "./pageCaret";
import { type Span, unsplit } from "./pageEdits";
import type { Formats } from "./pageFormats";
import type { PageHistory } from "./pageHistory";
import type { Laying } from "./pageLaying";
import { closing, leftBehind, opening, outdented, parted } from "./pageLists";
import { closedRun, unpaired } from "./pageMarks";
import { elementOf, ghostly, inCode, onSheet, unwritable } from "./pageNodes";
import type { Writer } from "./pageWriter";

const PAIRS: Record<string, string> = {
  formatBold: "**",
  formatItalic: "*",
  formatStrikeThrough: "~~",
  formatUnderline: "++",
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
      .then((said) => put({ ...one, insert: said }))
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
    const at = beside?.matches("hr[data-at]") ? spanned(beside.getAttribute("data-at")) : null;
    if (!beside || !at) return null;
    const until = spanned(beside.nextElementSibling?.getAttribute("data-at") ?? null)?.[0] ?? at[1];
    const text = hands.text();
    return { from: charsAt(text, at[0]), to: charsAt(text, until) };
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
    const broken = pastBreak(kind);
    if (broken) {
      const stays =
        kind === "deleteContentForward" ? (caret.caretSpan()?.from ?? broken.from) : broken.from;
      return put({ ...broken, insert: "" }, Math.min(stays, broken.from));
    }
    const [target] = event.getTargetRanges?.() ?? [];
    const range = target ?? caret.chosen();
    const picked = caret.chosen();
    const spread = Boolean(picked && !picked.collapsed);
    // A caret dropped on a table, a picture or a widget has no words under it to write among.
    if (range && !spread && unwritable(range.startContainer)) return;
    const held = picked ?? range;
    const begunNow =
      writer.kept?.lead !== undefined &&
      (writer.kept.offset < 0 || Boolean(held && ghostly(held.startContainer)));
    if (begunNow && kind.startsWith("delete")) return begun.dropBegun();
    const one =
      (!spread && begun.waited(kind)) ||
      (range && ((held && caret.unbegun(kind, held)) ?? caret.spanOfRange(range)));
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
      if (writer.pending?.at === one.from) return formats.pendingKey(one.from, typed);
      const opened = text.slice(text.lastIndexOf("\n", one.from - 1) + 1, one.from);
      if (typed === " " && /^\s*(?:>\s?)*\[ ?\]$/.test(opened)) {
        const box = `${opened.replace(/\[ ?\]$/, "")}- [ ] `;
        return put({ from: one.from - opened.length, to: one.from, insert: box });
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
    if (kind === "insertParagraph" && one.from === one.to && !inCode(range.startContainer)) {
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
      const at = one.from === one.to && !coded ? closedRun(text, one.from) : one.from;
      const insert = kind === "insertLineBreak" ? "\n" : parted(text, at, coded);
      return put({ from: at, to: Math.max(at, one.to), insert });
    }
    if (kind === "insertFromPaste") return pasted(one, event.dataTransfer);
    if (kind.startsWith("delete")) {
      const gone = unpaired(text, unsplit(text, one));
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
      })
      .catch(() => {});
  };

  return { asked, started, ended, pastedFiles };
};
