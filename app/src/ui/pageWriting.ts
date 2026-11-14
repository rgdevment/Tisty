import { SLASHED, type Slashed } from "tisty-editor/edit";
import { bytesAt, charsAt, spanned } from "tisty-editor/read";
import { t } from "../locales";
import { blank, blockPlaced, placed } from "./pageBlocks";
import { applied, changeOf, type Edit, linedAs, parted, unsplit } from "./pageEdits";
import { fieldAsked } from "./pageFields";
import { closing, joinedAt, leftBehind, nested, opening, outdented } from "./pageLists";
import {
  closedRun,
  closedTokens,
  closeOf,
  covered,
  openOf,
  pastClose,
  rejoined,
  trimmed,
  unpaired,
  worn,
} from "./pageMarks";
import {
  type Act,
  acted,
  gripped,
  type Moved,
  movedTo,
  roomy,
  swappedItem,
  topsOf,
  unitOf,
} from "./pageMoving";
import {
  FIELDS,
  ghostly,
  inCode,
  lineOfCaret,
  onSheet,
  opened,
  unwritable,
  WRITABLE,
} from "./pageNodes";
import { patched } from "./pagePatching";
import { endIn, endOf, GHOST, ghostAt, sourceAt, spotOf } from "./pageSpots";
import { tagList, tagMatch } from "./pageTags";
import { type Hands, type Kept, type Point, writerOf } from "./pageWriter";
import { unformattable } from "./quickly";

export type { Hands, Turns } from "./pageWriter";
export { turnsAnew } from "./pageWriter";

const BYTES = new TextEncoder();
// What the rest of the window reads of the page catches up once the keys rest, not on each one.
const RESTED_MS = 250;

export type PageWriting = {
  readonly drawn: () => void;
  readonly stop: () => void;
  readonly format: (open: string, close: string) => void;
  readonly formattable: () => boolean;
  readonly caretAt: () => number | null;
  readonly placeAt: (at: number) => void;
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

const PAIRS: Record<string, string> = {
  formatBold: "**",
  formatItalic: "*",
  formatStrikeThrough: "~~",
  formatUnderline: "++",
};
const KEYED: Record<string, readonly [string, string]> = {
  b: ["**", "**"],
  i: ["*", "*"],
  u: ["++", "++"],
  "shift x": ["~~", "~~"],
  "shift c": ["`", "`"],
  k: ["[", "]()"],
};

export const writtenOn = (page: Document, hands: Hands): PageWriting => {
  const writer = writerOf(page, hands);
  const { turns } = writer;
  let composing: { from: number; to: number } | null = null;
  const { undone, redone } = turns;
  // What was written in the Markdown since the page was left is one step back.
  if (turns.left !== undefined && turns.left !== hands.text()) {
    undone.push({ text: turns.left, at: changeOf(turns.left, hands.text()).from });
    redone.length = 0;
  }
  turns.left = undefined;

  // Until the page is drawn again it shows the words before the last key, so the caret kept in the words decides.
  const waited = (kind: string): { from: number; to: number } | null => {
    if (!writer.kept || writer.kept.lead !== undefined) return null;
    if (writer.kept.offset >= 0) {
      const caret = chosen();
      const here =
        caret?.collapsed &&
        caret.startContainer === writer.kept.node &&
        caret.startOffset === writer.kept.offset;
      if (!here || (kind !== "deleteContentBackward" && kind !== "deleteContentForward"))
        return null;
    }
    const { at } = writer.kept;
    const text = hands.text();
    const bare = text[at - 1] === "\n" && (at >= text.length || /[\r\n]/.test(text[at] ?? ""));
    if (kind === "deleteContentBackward" && bare) {
      let from = at - 1;
      while (from > 0 && /[\r\n]/.test(text[from - 1] ?? "")) from--;
      return { from, to: at };
    }
    if (kind.startsWith("deleteContentBackward") || kind.startsWith("deleteWordBackward"))
      return { from: Math.max(0, at - 1), to: at };
    if (kind.startsWith("delete")) return { from: at, to: Math.min(hands.text().length, at + 1) };
    return { from: at, to: at };
  };

  // A line begun and not written loses its stand-in on a press back, and the next key still begins it there.
  const dropBegun = () => {
    for (const one of page.querySelectorAll(GHOST)) one.remove();
    if (writer.kept) writer.kept = { ...writer.kept, node: page.body, offset: -1 };
  };

  // Out of a paragraph not yet written, going back leaves it and joins the block before.
  const unbegun = (kind: string, range: AbstractRange): { from: number; to: number } | null => {
    const unlaid = writer.kept?.lead !== undefined && writer.kept.offset < 0;
    if (!writer.kept || !range.collapsed || !(unlaid || ghostly(range.startContainer))) return null;
    const { at } = writer.kept;
    if (!kind.startsWith("delete") || kind.includes("Forward")) return { from: at, to: at };
    const text = hands.text();
    let from = at;
    while (from > 0 && /\s/.test(text[from - 1] ?? "")) from--;
    return { from, to: at };
  };

  const spanOfRange = (range: AbstractRange): { from: number; to: number } | null => {
    const text = hands.text();
    const same = writer.kept && range.collapsed && range.startContainer === writer.kept.node;
    if (same && writer.kept && range.startOffset === writer.kept.offset)
      return { from: writer.kept.at, to: writer.kept.at };
    const from = sourceAt(text, range.startContainer, range.startOffset);
    const to = range.collapsed ? from : sourceAt(text, range.endContainer, range.endOffset);
    if (from === null || to === null) return null;
    return { from: Math.min(from, to), to: Math.max(from, to) };
  };

  const chosen = (): AbstractRange | null => {
    const held = page.getSelection();
    return held?.rangeCount ? held.getRangeAt(0) : null;
  };

  let laying: Promise<void> = Promise.resolve();
  let resting: ReturnType<typeof setTimeout> | undefined;
  let unsaid: string | null = null;
  const told = () => {
    clearTimeout(resting);
    if (unsaid !== null) hands.shown(unsaid);
    unsaid = null;
  };
  const rested = (now: string) => {
    clearTimeout(resting);
    unsaid = now;
    resting = setTimeout(told, RESTED_MS);
  };

  // Each change is laid in turn: the core draws again the blocks around it and only those change on the page.
  const laidOut = (was: string, now: string, one: Edit) => {
    // A task's page is small and dressed with its hints and buttons, which only a whole drawing lays again.
    if (page.documentElement.classList.contains("untitled")) writer.whole = true;
    const from = bytesAt(now, one.from);
    const added = BYTES.encode(one.insert).length;
    const by = added - BYTES.encode(was.slice(one.from, one.to)).length;
    laying = laying.then(async () => {
      unsaid = null;
      clearTimeout(resting);
      if (writer.whole) return hands.redraw(hands.text());
      const near = await hands.near(now, from, from + added).catch(() => null);
      const sheet = page.querySelector(".page");
      const laid = sheet && near && patched(sheet, near, by);
      if (!sheet || !laid) {
        writer.whole = true;
        return hands.redraw(hands.text());
      }
      hands.dressed(sheet, laid);
      rested(now);
      if (hands.text() === now) drawn();
    });
  };

  const put = (asked: Edit, at = asked.from + asked.insert.length, drawnWhole = false) => {
    const text = hands.text();
    const [one, caret] = linedAs(text, asked, at);
    const next = applied(text, one);
    if (next === text) return;
    const stood =
      writer.kept && writer.kept.offset < 0 ? writer.kept.at : (caretSpan()?.from ?? one.from);
    undone.push({ text, at: Math.min(stood, text.length), after: caret });
    redone.length = 0;
    writer.kept = { node: page.body, offset: -1, at: caret };
    hands.write(next);
    if (drawnWhole) writer.whole = true;
    laidOut(text, next, one);
  };

  const turned = (from: typeof undone, to: typeof undone, forward = false) => {
    const back = from.pop();
    if (!back) return;
    writer.pending = null;
    to.push({ text: hands.text(), at: back.at, after: back.after });
    writer.kept = { node: page.body, offset: -1, at: forward ? (back.after ?? back.at) : back.at };
    hands.write(back.text);
    writer.whole = true;
    hands.redraw(back.text);
  };

  const pasted = (one: { from: number; to: number }, data: DataTransfer | null) => {
    const html = data?.getData("text/html") || null;
    const plain = data?.getData("text/plain") ?? "";
    hands
      .translated(html, plain)
      .then((said) => put({ ...one, insert: said }))
      .catch(() => put({ ...one, insert: plain }));
  };

  // A slash opens the blocks where a word could start, as it always did; inside code it is only a slash.
  const slashes = (text: string, one: { from: number; to: number }, range: AbstractRange) =>
    one.from === one.to &&
    !inCode(range.startContainer) &&
    (blank(text, one.from) || /(^|\s)$/.test(text.slice(0, one.from).split("\n").pop() ?? ""));

  // A page break holds no words, so the key that would join a block to it takes the break away instead.
  const pastBreak = (kind: string): { from: number; to: number } | null => {
    const back = kind === "deleteContentBackward";
    if (!back && kind !== "deleteContentForward") return null;
    const range = chosen();
    if (!range?.collapsed) return null;
    const node = range.startContainer;
    const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
    const top = element?.closest(".page > [data-at]");
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
    if (kind === "historyUndo") return turned(undone, redone);
    if (kind === "historyRedo") return turned(redone, undone, true);
    const broken = pastBreak(kind);
    if (broken) {
      const stays =
        kind === "deleteContentForward" ? (caretSpan()?.from ?? broken.from) : broken.from;
      return put({ ...broken, insert: "" }, Math.min(stays, broken.from));
    }
    const [target] = event.getTargetRanges?.() ?? [];
    const range = target ?? chosen();
    const picked = chosen();
    const spread = Boolean(picked && !picked.collapsed);
    // A caret dropped on a table, a picture or a widget has no words under it to write among.
    if (range && !spread && unwritable(range.startContainer)) return;
    const held = picked ?? range;
    const begunNow =
      writer.kept?.lead !== undefined &&
      (writer.kept.offset < 0 || Boolean(held && ghostly(held.startContainer)));
    if (begunNow && kind.startsWith("delete")) return dropBegun();
    const one =
      (!spread && waited(kind)) || (range && ((held && unbegun(kind, held)) ?? spanOfRange(range)));
    if (!range || !one) return;
    const text = hands.text();
    const typed = event.data ?? event.dataTransfer?.getData("text/plain") ?? "";
    const begun = begunNow ? writer.kept : null;
    if (begun && kind !== "insertText" && kind !== "insertReplacementText") return;
    if (begun && kind === "insertText" && typed === "/") return hands.blocks(caretPoint(), true);
    if (begun) return firstKey(begun, typed);
    if (kind === "insertText" && typed === "/" && slashes(text, one, range))
      return hands.blocks(caretPoint(), true);
    if (kind === "insertText" && one.from === one.to) {
      if (writer.pending?.at === one.from) return pendingKey(one.from, typed);
      const opened = text.slice(text.lastIndexOf("\n", one.from - 1) + 1, one.from);
      if (typed === " " && /^\s*(?:>\s?)*\[ ?\]$/.test(opened)) {
        const box = `${opened.replace(/\[ ?\]$/, "")}- [ ] `;
        return put({ from: one.from - opened.length, to: one.from, insert: box });
      }
      const carry = writer.carried?.caret === one.from ? writer.carried : null;
      writer.carried = null;
      if (carry && !typed.trim()) {
        writer.carried = { ...carry, caret: one.from + typed.length };
        return put({ ...one, insert: typed });
      }
      if (carry) {
        const spaces = text.slice(carry.from + carry.close.length, one.from);
        return put(
          { from: carry.from, to: one.from, insert: `${spaces}${typed}${carry.close}` },
          carry.from + spaces.length + typed.length,
        );
      }
      const past = typed.trim() ? null : pastClose(text, one.from);
      if (past !== null) {
        writer.carried = {
          from: one.from,
          close: text.slice(one.from, past),
          caret: past + typed.length,
        };
        return put({ from: past, to: past, insert: typed });
      }
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
    if (pair) format(pair, pair);
  };

  // A piece drawn whole, as code or a tag, takes no keys, so a caret dropped inside it stands right after it.
  const outOfSolid = (range: AbstractRange) => {
    const node = range.startContainer;
    const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
    const atom = element?.closest('.page [data-src][contenteditable="false"]');
    if (!atom?.parentElement || atom.parentElement.closest('[contenteditable="false"]')) return;
    const early = node.nodeType === Node.TEXT_NODE && range.startOffset * 2 < (node as Text).length;
    const at = Array.prototype.indexOf.call(atom.parentElement.childNodes, atom);
    const [holder, offset] = early ? [atom.parentElement, at] : endOf(atom);
    page.getSelection()?.collapse(holder, offset);
  };

  const noted = () => {
    besideLine();
    const range = chosen();
    const sheet = page.querySelector(".page");
    if (range && !range.collapsed && sheet?.contains(range.startContainer))
      writer.lastChosen = spanOfRange(range);
    else if (range?.collapsed) {
      writer.lastChosen = null;
      outOfSolid(range);
    }
    if (writer.leftFrom === undefined) return;
    if (blockStart() !== writer.leftFrom) hands.settle?.();
    writer.leftFrom = undefined;
  };

  const chosenNow = () => {
    const range = chosen();
    const one = range && !range.collapsed ? spanOfRange(range) : writer.lastChosen;
    return one && one.from < one.to ? one : null;
  };

  const linked = (one: { from: number; to: number }) => {
    const held = page.getSelection();
    const box = held?.rangeCount ? held.getRangeAt(0).getBoundingClientRect() : null;
    const near = box ?? page.querySelector(".page")?.getBoundingClientRect();
    if (!near) return;
    void fieldAsked(page, near, t("linkAddress"), { kind: "url" }).then((to) => {
      if (!to) return;
      const inner = hands.text().slice(one.from, one.to) || to;
      put({ ...one, insert: `[${inner}](${to})` });
    });
  };

  const format = (open: string, close: string) => {
    if (unformattable(page)) return;
    const carry = writer.carried;
    writer.carried = null;
    const text = hands.text();
    const chosenSpan = trimmed(text, chosenNow());
    const one = chosenSpan && close !== "]()" ? covered(text, chosenSpan, open) : chosenSpan;
    if (!one && carry && carry.caret === caretSpan()?.from && carry.close.startsWith(close)) return;
    if (close === "]()") {
      const span = one ?? caretSpan();
      if (span) linked(span);
      return;
    }
    if (!one) return waitFor(open, close);
    writer.pending = null;
    const change = worn(text, one, open, close);
    put(change, change.from + change.insert.length);
  };

  const waitFor = (open: string, close: string) => {
    const at = caretSpan()?.from;
    if (at === undefined) return;
    const text = hands.text();
    const marks = writer.pending?.at === at ? writer.pending.marks : [];
    if (marks.some(([one]) => one === open)) {
      const rest = marks.filter(([one]) => one !== open);
      writer.pending = rest.length ? { at, marks: rest } : null;
      return;
    }
    const closers = closedTokens(text, at);
    const shut = closers.findIndex(
      (one) => one === close || (open === "==" && one.startsWith("==")),
    );
    if (shut >= 0) {
      const passed = closers.slice(0, shut + 1);
      const reopened = passed
        .slice(0, shut)
        .reverse()
        .map((one) => [one.startsWith("==") ? "==" : one, one] as const);
      const past = at + passed.join("").length;
      const next = [...reopened, ...marks];
      writer.kept = { node: page.body, offset: -1, at: past };
      writer.pending = next.length ? { at: past, marks: next } : null;
      return;
    }
    writer.pending = { at, marks: [...marks, [open, close]] };
    focused();
  };

  const pendingKey = (at: number, typed: string) => {
    const wait = writer.pending;
    if (!wait) return;
    if (!typed.trim()) {
      put({ from: at, to: at, insert: typed });
      writer.pending = { ...wait, at: at + typed.length };
      return;
    }
    writer.pending = null;
    const lead = /^\s*/.exec(typed)?.[0] ?? "";
    const words = typed.trim();
    const trail = typed.slice(lead.length + words.length);
    const [from, marks] = lead ? [at, wait.marks] : rejoined(hands.text(), at, wait.marks);
    const left = { at: from, marks };
    const open = openOf(left);
    put(
      { from, to: from, insert: `${lead}${open}${words}${closeOf(left)}${trail}` },
      from + lead.length + open.length + words.length,
    );
  };

  const started = (event: CompositionEvent) => {
    if (!onSheet(page, event)) return;
    const range = chosen();
    composing = range && spanOfRange(range);
  };
  const ended = (event: CompositionEvent) => {
    if (!onSheet(page, event)) return;
    const one = composing;
    composing = null;
    writer.typing = true;
    if (writer.kept?.lead !== undefined && one?.from === writer.kept.at)
      return firstKey(writer.kept, event.data ?? "");
    if (one && one.from === one.to && writer.pending?.at === one.from)
      return pendingKey(one.from, event.data ?? "");
    if (one) put({ ...one, insert: event.data ?? "" });
  };

  // A collapsible's name is drawn in its summary with no place marked in the text, so the caret is counted there.
  const summaryAt = (found: [Node, number], at: number): [Node, number] | null => {
    const node = found[0];
    const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
    const box = element?.closest("details[data-at]");
    const summary = box?.querySelector(":scope > summary");
    const span = spanned(box?.getAttribute("data-at") ?? null);
    if (!summary || !span) return null;
    const text = hands.text();
    const start = charsAt(text, span[0]);
    const ends = text.indexOf("\n", start);
    const line = text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "");
    const name = start + (/^:::\s*details\s?/.exec(line)?.[0].length ?? line.length);
    if (at < name || at > start + line.length) return null;
    const words = summary.firstChild;
    if (words?.nodeType !== Node.TEXT_NODE) return [summary, 0];
    return [words, Math.min(at - name, (words as Text).length)];
  };

  const caretLaid = () => {
    if (!writer.kept) return;
    const unwritten = writer.kept.lead !== undefined || !spotOf(page, hands.text(), writer.kept.at);
    const ghost = unwritten ? ghostAt(page, hands.text(), writer.kept.at) : null;
    const found: [Node, number] | null = ghost
      ? [ghost, 0]
      : spotOf(page, hands.text(), writer.kept.at);
    if (!found) return;
    focused();
    const spot = summaryAt(found, writer.kept.at) ?? found;
    // A block drawn empty, as a heading just begun, holds no line for the caret, so it gets one.
    const [holder] = spot;
    const bare = holder.nodeType === Node.ELEMENT_NODE ? (holder as Element) : null;
    if (bare && !bare.textContent && !bare.querySelector("br"))
      bare.append(page.createElement("br"));
    const held = page.getSelection();
    held?.collapse(spot[0], spot[1]);
    writer.kept = { ...writer.kept, node: spot[0], offset: spot[1] };
    inSight(holder);
  };

  // A caret laid by a script never scrolls, so the line being written is brought back when it slips out of sight.
  const inSight = (holder: Node) => {
    const view = page.defaultView;
    const held = page.getSelection();
    if (!view || !page.hasFocus() || !held?.rangeCount) return;
    const range = held.getRangeAt(0);
    const line = holder.nodeType === Node.ELEMENT_NODE ? (holder as Element) : holder.parentElement;
    let box =
      typeof range.getBoundingClientRect === "function" ? range.getBoundingClientRect() : null;
    if (!box?.height && line) box = line.getBoundingClientRect();
    if (!box) return;
    const room = 48;
    if (box.bottom > view.innerHeight - room)
      view.scrollBy({ top: box.bottom - view.innerHeight + room });
    else if (box.top < room / 2) view.scrollBy({ top: box.top - room / 2 });
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

  const drawn = () => {
    writer.whole = false;
    opened(page);
    caretLaid();
    offerTags();
    gripping.rest();
    besideLine();
  };

  // Colours, icons and cards land after a block is drawn and take the place of its words, the caret with them.
  const Watching = page.defaultView?.MutationObserver;
  const watching =
    Watching &&
    new Watching((records) => {
      if (records.some((one) => one.type === "attributes")) opened(page);
      if (writer.kept && writer.kept.offset >= 0 && !writer.kept.node.isConnected) caretLaid();
    });
  watching?.observe(page.body, { childList: true, subtree: true });
  watching?.observe(page.documentElement, { attributes: true, attributeFilter: ["class"] });

  // A line begun from a hint, a label or a button takes what it opens with and its blank lines with its first key.
  const firstKey = (begun: Kept, typed: string) => {
    const lead = begun.lead ?? "";
    const wait = writer.pending?.at === begun.at && typed.trim() ? writer.pending : null;
    writer.pending = null;
    const open = wait ? openOf(wait) : "";
    put(
      {
        from: begun.at,
        to: begun.at,
        insert: `${lead}${open}${typed}${wait ? closeOf(wait) : ""}${begun.tail ?? ""}`,
      },
      begun.at + lead.length + open.length + typed.length,
    );
  };

  const laidGhost = (near: Element, inside: boolean, at: number, lead: string, tail: string) => {
    for (const one of page.querySelectorAll(GHOST)) if (one !== near) one.remove();
    const titled = near.matches(".page > h1[data-title]");
    const ghost = titled ? (near as HTMLElement) : page.createElement("p");
    ghost.dataset.ghost = "";
    if (!titled) {
      ghost.append(page.createElement("br"));
      if (inside) near.append(ghost);
      else near.after(ghost);
    }
    writer.kept = { node: ghost, offset: 0, at, lead, tail };
    page.querySelector<HTMLElement>(".page")?.focus();
    page.getSelection()?.collapse(ghost, 0);
  };

  const begin = (asked: number, seed: string, near: Element) => {
    const text = hands.text();
    const at = joinedAt(text, asked, seed);
    laidGhost(near, false, at, `${opening(text, at, seed)}${seed}`, closing(text, at));
  };

  // A quote or a sticky note with no words yet holds no caret, so its first line is begun inside it.
  const quoted = (event: MouseEvent) => {
    const quote = (event.target as Element | null)?.closest?.(".page > blockquote[data-at]");
    if (!quote || quote.querySelector(`[data-src], ${GHOST}`)) return;
    const span = spanned(quote.getAttribute("data-at"));
    if (!span) return;
    const text = hands.text();
    let at = charsAt(text, span[1]);
    while (at > 0 && /[\r\n]/.test(text[at - 1] ?? "")) at--;
    const line = text.slice(text.lastIndexOf("\n", at - 1) + 1, at);
    const bare = /^\s*>\s*$/.test(line);
    laidGhost(quote, true, at, bare ? (line.endsWith(" ") ? "" : " ") : "\n> ", "");
  };

  // A section's hint is a comment the page does not show as words, so it is mended in a field of its own.
  const hintWords = (at: [number, number], hint: HTMLElement) => {
    const text = hands.text();
    const from = charsAt(text, at[0]);
    const to = charsAt(text, at[1]);
    const box = /^<!--\s*(-\s*\[ \]\s*)/.exec(text.slice(from, to))?.[1] ?? "";
    const near = hint.getBoundingClientRect();
    const value = hint.dataset.hint ?? "";
    void fieldAsked(page, near, t("sectionHint"), { value }).then((said) => {
      if (said !== null) put({ from, to, insert: `<!-- ${box}${said} -->` });
    });
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

  const caretSpan = () => {
    if (writer.kept && writer.kept.offset < 0) return { from: writer.kept.at, to: writer.kept.at };
    const range = chosen();
    return range && (unbegun("insertText", range) ?? spanOfRange(range));
  };

  const blockSpan = () => {
    const held = page.getSelection();
    const node = held?.anchorNode ?? null;
    const element = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
    return spanned(element?.closest(".page > [data-at]")?.getAttribute("data-at") ?? null);
  };

  const blockEnd = (at: number) => {
    const span = blockSpan();
    return span ? charsAt(hands.text(), span[1]) : at;
  };

  // Only a paragraph is turned whole, from its first line, as a line broken inside it still belongs to it.
  const paragraphStart = () => {
    const held = page.getSelection();
    const node = held?.anchorNode ?? null;
    const element = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
    const span = spanned(element?.closest(".page > p[data-at]")?.getAttribute("data-at") ?? null);
    return span ? charsAt(hands.text(), span[0]) : null;
  };

  const slashed = (one: Slashed, drawnWhole = false): Edit | null => {
    if (
      writer.kept?.lead !== undefined &&
      writer.kept.at > 0 &&
      hands.text()[writer.kept.at - 1] !== "\n"
    )
      firstKey(writer.kept, "");
    const span = caretSpan();
    if (!span) return null;
    const text = hands.text();
    const end = Math.max(span.to, blockEnd(span.to));
    const how = placed(text, span.from, end, one, paragraphStart());
    if ("wrap" in how) {
      format(how.wrap, how.close);
      return null;
    }
    if ("edit" in how) {
      put(how.edit, how.caret, drawnWhole);
      return how.edit;
    }
    const first = !text.slice(0, span.from).trim();
    hands.ask(
      how.ask,
      caretPoint(),
      (said) => {
        const now = hands.text();
        const at = Math.min(span.from, now.length);
        const laid = how.block
          ? blockPlaced(now, at, Math.min(end, now.length), said, said.length)
          : { edit: { from: at, to: at, insert: said }, caret: at + said.length };
        put(laid.edit, laid.caret);
        return true;
      },
      first,
    );
    return null;
  };

  // The blocks wait on the sheet's left edge, beside the line being written, as the folded bar did.
  const plus = page.createElement("button");
  plus.type = "button";
  plus.className = "page-plus";
  plus.textContent = "+";
  plus.title = t("blocksOf");
  plus.setAttribute("aria-label", t("blocksOf"));
  plus.hidden = true;
  plus.addEventListener("mousedown", (event) => event.preventDefault());
  plus.addEventListener("click", () => hands.blocks(caretPoint(), false));
  page.body.append(plus);

  const placedBeside = () => {
    const held = page.getSelection();
    const range = held?.rangeCount ? held.getRangeAt(0) : null;
    const sheet = page.querySelector<HTMLElement>(".page");
    const field = page.activeElement?.closest?.(FIELDS);
    const node = range?.startContainer;
    const element = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
    const tabled = element?.closest(".page > .table");
    if (!range || !sheet?.contains(range.startContainer) || field || tabled || !roomy(page)) {
      plus.hidden = true;
      return;
    }
    const ghost = page.querySelector(GHOST);
    let box = range.getBoundingClientRect();
    if (box.width === 0 && box.height === 0) {
      const line = ghost ?? lineOfCaret(range);
      if (!line) {
        plus.hidden = true;
        return;
      }
      box = line.getBoundingClientRect();
    }
    const view = page.defaultView;
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const edge = sheet.getBoundingClientRect().left;
    plus.hidden = false;
    plus.style.top = `${(box.top + (view?.scrollY ?? 0)) / zoom}px`;
    plus.style.left = `${(edge + (view?.scrollX ?? 0)) / zoom + 18}px`;
  };

  // A block just drawn is measured at its placeholder size until the next frame lays it out.
  let measuring = 0;
  const besideLine = () => {
    placedBeside();
    const view = page.defaultView;
    if (!view) return;
    view.cancelAnimationFrame(measuring);
    measuring = view.requestAnimationFrame(() => {
      placedBeside();
      measuring = view.requestAnimationFrame(placedBeside);
    });
  };

  // The caret goes with the words it stood in, as far into them as it was.
  // Words changed from beside the page, as a table's cell, are laid block by block and leave the caret where it is.
  const keptAcross = (one: Edit) => {
    const held = chosen();
    const sheet = page.querySelector(".page");
    const was = held?.collapsed && sheet?.contains(held.startContainer) ? caretSpan()?.from : null;
    const by = one.insert.length - (one.to - one.from);
    writer.kept =
      was === null || was === undefined
        ? null
        : { node: page.body, offset: -1, at: was <= one.from ? was : Math.max(one.from, was + by) };
    writer.pending = null;
  };

  const rewrite = (next: string) => {
    const text = hands.text();
    if (next === text) return;
    const one = changeOf(text, next);
    undone.push({ text, at: one.from });
    redone.length = 0;
    keptAcross(one);
    hands.write(next);
    laidOut(text, next, one);
  };

  // Words from another hand are not the person's to take back, so what Ctrl+Z held before them is let go.
  const outside = (was: string, next: string) => {
    if (next === was) return;
    const one = changeOf(was, next);
    undone.length = 0;
    redone.length = 0;
    keptAcross(one);
    laidOut(was, next, one);
  };

  const replaced = (moved: Moved | null) => {
    if (!moved) return;
    const at = caretSpan()?.from ?? -1;
    const inside = at >= moved.was && at <= moved.was + moved.size;
    put(changeOf(hands.text(), moved.text), moved.caret + (inside ? at - moved.was : 0));
  };

  const unitActed = (top: Element, act: Act) => {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    const tops = topsOf(sheet);
    replaced(acted(hands.text(), tops, tops.indexOf(top), act));
  };

  const movedOnto = (top: Element, target: Element | null) => {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    const text = hands.text();
    const tops = topsOf(sheet);
    const unit = unitOf(text, tops, tops.indexOf(top));
    const at = spanned(target?.getAttribute("data-at") ?? null);
    if (unit) replaced(movedTo(text, unit, at ? charsAt(text, at[0]) : text.length));
  };

  // Alt with an arrow moves the item or the block the caret stands in, as it did in the Markdown.
  const stepped = (event: KeyboardEvent) => {
    const way = event.key === "ArrowUp" ? -1 : 1;
    const node = page.getSelection()?.anchorNode ?? null;
    const element = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
    const sheet = page.querySelector(".page");
    if (!element || !sheet) return;
    event.preventDefault();
    const item = element.closest("li[data-at]");
    if (item && sheet.contains(item)) return replaced(swappedItem(hands.text(), item, way));
    const top = element.closest(".page > [data-at]");
    if (top) unitActed(top, way < 0 ? "up" : "down");
  };

  const broke = (event: KeyboardEvent) => {
    event.preventDefault();
    hands.settle?.();
    const range = chosen();
    const rule = SLASHED.find((one) => one.key === "rule");
    if (!range || !rule || inCode(range.startContainer)) return;
    const laid = slashed(rule);
    if (!laid) return;
    const text = hands.text();
    let at =
      writer.kept && writer.kept.offset < 0
        ? writer.kept.at
        : Math.min(text.length, laid.from + laid.insert.replace(/\s+$/, "").length);
    while (at > 0 && /[\r\n]/.test(text[at - 1] ?? "")) at--;
    writer.kept = {
      node: page.body,
      offset: -1,
      at,
      lead: opening(text, at, ""),
      tail: closing(text, at),
    };
  };

  // Tab nests a list item under the one above and Shift+Tab takes it back out; in code it indents.
  const tabbed = (event: KeyboardEvent) => {
    event.preventDefault();
    const at = caretSpan()?.from;
    const range = chosen();
    if (at === undefined || !range) return;
    if (inCode(range.startContainer)) {
      if (!event.shiftKey) put({ from: at, to: at, insert: "  " });
      return;
    }
    const moved = nested(hands.text(), at, event.shiftKey);
    if (moved) put(moved.edit, moved.caret);
  };

  const blockOfCaret = () => {
    const node = chosen()?.startContainer;
    const element = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
    return element?.closest(".page [data-at]") ?? null;
  };

  // A block is told apart by where it starts, since each key draws it again as another element.
  const blockStart = () => blockOfCaret()?.getAttribute("data-at")?.split("-")[0] ?? null;

  const laidAt = (spot: [Node, number]) => {
    page.getSelection()?.collapse(spot[0], spot[1]);
    inSight(spot[0]);
  };

  // A tag or other piece drawn whole at a line's end leaves the browser no place after it, so End is placed here.
  const toEnd = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey) {
      const last = Array.from(page.querySelectorAll(".page [data-at]"))
        .filter((one) => one.closest(WRITABLE) && !one.closest('[contenteditable="false"]'))
        .pop();
      if (!last) return;
      event.preventDefault();
      laidAt(endIn(last));
      return;
    }
    const from = blockOfCaret();
    if (!from) return;
    setTimeout(() => {
      if (blockOfCaret() !== from && from.isConnected) laidAt(endIn(from));
    });
  };

  const keyed = (event: KeyboardEvent) => {
    if (!onSheet(page, event)) return;
    if (event.key === "End" && !event.shiftKey && !event.altKey) return toEnd(event);
    if (event.key === "Tab" && !event.ctrlKey && !event.metaKey && !event.altKey)
      return tabbed(event);
    if (event.altKey && (event.key === "ArrowUp" || event.key === "ArrowDown"))
      return stepped(event);
    if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
    if (event.key === "Enter") return broke(event);
    const named = `${event.shiftKey ? "shift " : ""}${event.key.toLowerCase()}`;
    if (named === "z" || named === "shift z" || named === "y") {
      event.preventDefault();
      writer.pending = null;
      writer.carried = null;
      return named === "z" ? turned(undone, redone) : turned(redone, undone, true);
    }
    const pair = KEYED[named];
    if (pair) {
      event.preventDefault();
      return format(pair[0], pair[1]);
    }
    if (event.key !== "/") return;
    event.preventDefault();
    hands.blocks(caretPoint(), false);
  };

  const tags = tagList(page, (match, tag) => {
    writer.typing = false;
    const text = hands.text();
    const to = Math.min(match.to, text.length);
    put({ from: Math.min(match.from, to), to, insert: `#${tag}` });
  });

  const offerTags = () => {
    const at = writer.kept?.at;
    const range = chosen();
    const known = hands.tags;
    const open =
      writer.typing && at !== undefined && known && !(range && inCode(range.startContainer));
    tags.offer(open && known ? tagMatch(hands.text(), at, known()) : null);
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
    const span = caretSpan();
    if (!span) return;
    const end = Math.max(span.to, blockEnd(span.to));
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

  const gripping = gripped(page, { moved: movedOnto, asked: (top, at) => hands.acts(at, top) });

  // The page's events come from its own window, so a key is told by what it carries, not by its class.
  const ticking = (event: MouseEvent) => {
    if ((event.target as Element | null)?.closest?.(".page .check, .page input[type=checkbox]"))
      event.preventDefault();
  };

  const moved = (event: Event) => {
    const key = "key" in event ? String(event.key) : null;
    if (key !== null && !key.startsWith("Arrow") && !/^(Home|End|Page)/.test(key)) return;
    if ((event.target as Element | null)?.closest?.(".tag-list")) return;
    const held = event as KeyboardEvent;
    const across = key !== null && Boolean(held.ctrlKey || held.metaKey);
    if (key === null || /^(ArrowUp|ArrowDown|Page)/.test(key) || across)
      writer.leftFrom = blockStart();
    writer.kept = null;
    writer.pending = null;
    writer.carried = null;
    writer.typing = false;
    tags.hide();
  };

  page.addEventListener("beforeinput", asked);
  page.addEventListener("compositionstart", started);
  page.addEventListener("compositionend", ended);
  page.addEventListener("mousedown", moved);
  page.addEventListener("click", quoted);
  page.addEventListener("mousedown", ticking);
  page.addEventListener("keydown", moved);
  page.addEventListener("selectionchange", noted);
  page.addEventListener("keydown", keyed);
  page.addEventListener("paste", pastedFiles);
  const away = () => hands.settle?.();
  page.defaultView?.addEventListener("blur", away);
  opened(page);
  return {
    drawn,
    format,
    formattable: () => !unformattable(page),
    caretAt: () => caretSpan()?.from ?? null,
    placeAt: (at: number) => {
      writer.kept = { node: page.body, offset: -1, at: Math.min(at, hands.text().length) };
      page.querySelector<HTMLElement>(".page")?.focus({ preventScroll: true });
      caretLaid();
      const node = writer.kept?.node;
      const holder = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
      holder?.scrollIntoView?.({ block: "nearest" });
    },
    rewrite,
    outside,
    chosen: () => {
      const one = chosenNow();
      return one ? hands.text().slice(one.from, one.to) : null;
    },
    slashed,
    unitActed,
    begin,
    hintWords,
    typed: (said) => {
      if (writer.kept?.lead !== undefined) return firstKey(writer.kept, said);
      const span = caretSpan();
      if (span) put({ ...span, insert: said });
    },
    cut: () => {
      const one = chosenNow();
      if (one) put({ ...one, insert: "" }, one.from);
    },
    stop: () => {
      turns.left = hands.text();
      told();
      hands.settle?.();
      page.defaultView?.removeEventListener("blur", away);
      watching?.disconnect();
      page.removeEventListener("selectionchange", noted);
      page.removeEventListener("keydown", keyed);
      page.removeEventListener("paste", pastedFiles);
      tags.stop();
      plus.remove();
      gripping.stop();
      page.removeEventListener("mousedown", moved);
      page.removeEventListener("click", quoted);
      page.removeEventListener("mousedown", ticking);
      page.removeEventListener("keydown", moved);
      page.removeEventListener("beforeinput", asked);
      page.removeEventListener("compositionstart", started);
      page.removeEventListener("compositionend", ended);
      page.querySelector<HTMLElement>(".page")?.removeAttribute("contenteditable");
    },
  };
};
