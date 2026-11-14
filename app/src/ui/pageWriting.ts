import type { Asking, Slashed } from "tisty-editor/edit";
import { bytesAt, charsAt, spanned } from "tisty-editor/read";
import { t } from "../locales";
import type { PlacedNear } from "../placing";
import { blank, blockPlaced, placed } from "./pageBlocks";
import { fieldAsked } from "./pageFields";
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
import { patched } from "./pagePatching";
import { GHOST, ghostAt, sourceAt, spotOf } from "./pageSpots";
import { unformattable } from "./quickly";

type Edit = { readonly from: number; readonly to: number; readonly insert: string };
/** `lead` and `tail` wrap the first key of a line begun from a hint or a button, written only once a key comes. */
type Kept = {
  readonly node: Node;
  readonly offset: number;
  readonly at: number;
  readonly lead?: string;
  readonly tail?: string;
};

export type Hands = {
  readonly text: () => string;
  readonly write: (next: string) => void;
  readonly shown: (now: string) => void;
  readonly redraw: (now: string) => void;
  readonly near: (body: string, from: number, to: number) => Promise<PlacedNear>;
  readonly dressed: (sheet: Element, laid: readonly Element[]) => void;
  readonly translated: (html: string | null, plain: string) => Promise<string>;
  readonly ask: Asking["ask"];
  readonly blocks: (at: { x: number; y: number }, slash: boolean) => void;
  readonly acts: (at: { x: number; y: number }, top: Element) => void;
};

type Point = { readonly x: number; readonly y: number };

const BYTES = new TextEncoder();
// What the rest of the window reads of the page catches up once the keys rest, not on each one.
const RESTED_MS = 250;

export type PageWriting = {
  readonly drawn: () => void;
  readonly stop: () => void;
  readonly format: (open: string, close: string) => void;
  readonly formattable: () => boolean;
  readonly rewrite: (next: string) => void;
  readonly chosen: () => string | null;
  readonly cut: () => void;
  readonly slashed: (one: Slashed, drawnWhole?: boolean) => Edit | null;
  readonly typed: (said: string) => void;
  readonly unitActed: (top: Element, act: Act) => void;
  readonly begin: (at: number, seed: string, near: Element) => void;
  readonly hintWords: (at: [number, number], hint: HTMLElement) => void;
};

const FIELDS = "input, textarea, select";

const WRITABLE =
  ".page > :is(p, h1, h2, h3, h4, h5, h6, ul, ol, blockquote, figure.code, .footnote)";
const PAIRS: Record<string, string> = {
  formatBold: "**",
  formatItalic: "*",
  formatStrikeThrough: "~~",
  formatUnderline: "++",
};
const QUOTED = /^(\s*(?:>\s?)*)/;
const LISTED = /^(\s*(?:>\s?)*\s*)([-*+]|(\d+)([.)]))(\s+)(\[[ xX]\]\s+)?/;

const lineOf = (text: string, at: number) => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const end = text.indexOf("\n", at);
  return text.slice(start, end < 0 ? text.length : end);
};

// A new paragraph keeps what the line it leaves stands in: the quote, the list and its kind of item.
const parted = (text: string, at: number, inCode: boolean): string => {
  if (inCode) return "\n";
  const line = lineOf(text, at);
  const item = LISTED.exec(line);
  if (item) {
    const [, quote, , number, close, gap, box] = item;
    const mark = number ? `${Number(number) + 1}${close}` : item[2];
    return `\n${quote}${mark}${gap}${box ? "[ ] " : ""}`;
  }
  const quote = QUOTED.exec(line)?.[1] ?? "";
  return quote ? `\n${quote.trimEnd()}\n${quote}` : "\n\n";
};

// A change made whole, as a block moved, is laid as the one stretch of words it altered.
const changeOf = (was: string, now: string): Edit => {
  const most = Math.min(was.length, now.length);
  let start = 0;
  while (start < most && was[start] === now[start]) start++;
  let end = 0;
  while (end < most - start && was[was.length - 1 - end] === now[now.length - 1 - end]) end++;
  return { from: start, to: was.length - end, insert: now.slice(start, now.length - end) };
};

// Blank lines a begun line needs before it: one to stay in the list above, two to stand apart.
const LIST_ITEM = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])\s/;

const opening = (text: string, at: number, seed: string) => {
  const before = text.slice(0, at);
  if (!before.trim()) return "";
  const lines = before.replace(/\s+$/, "").split("\n");
  const joins = LIST_ITEM.test(seed) && LIST_ITEM.test(lines[lines.length - 1] ?? "");
  const have = /\n*$/.exec(before)?.[0].length ?? 0;
  return "\n".repeat(Math.max(0, (joins ? 1 : 2) - have));
};

const closing = (text: string, at: number) => {
  const after = text.slice(at);
  if (!after.trim()) return "";
  return "\n".repeat(Math.max(0, 2 - (/^\n*/.exec(after)?.[0].length ?? 0)));
};

const applied = (text: string, one: Edit) =>
  `${text.slice(0, one.from)}${one.insert}${text.slice(one.to)}`;

export const writtenOn = (page: Document, hands: Hands): PageWriting => {
  let kept: Kept | null = null;
  let composing: { from: number; to: number } | null = null;
  const undone: { text: string; at: number }[] = [];
  const redone: { text: string; at: number }[] = [];

  // Until the page is drawn again it shows the words before the last key, so the caret kept in the words decides.
  const waited = (kind: string): { from: number; to: number } | null => {
    if (!kept || kept.offset >= 0) return null;
    const { at } = kept;
    if (kind.startsWith("deleteContentBackward") || kind.startsWith("deleteWordBackward"))
      return { from: Math.max(0, at - 1), to: at };
    if (kind.startsWith("delete")) return { from: at, to: Math.min(hands.text().length, at + 1) };
    return { from: at, to: at };
  };

  const ghostly = (node: Node) =>
    Boolean(
      (node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement)?.closest(
        GHOST,
      ),
    );

  const unwritable = (node: Node) =>
    Boolean(
      (node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement)?.closest(
        '.page > [contenteditable="false"]',
      ),
    );

  // Out of a paragraph not yet written, going back leaves it and joins the block before.
  const unbegun = (kind: string, range: AbstractRange): { from: number; to: number } | null => {
    if (!kept || !range.collapsed || !ghostly(range.startContainer)) return null;
    const { at } = kept;
    if (kept.lead !== undefined && kind.startsWith("delete")) {
      for (const one of page.querySelectorAll(GHOST)) one.remove();
      kept = null;
      return null;
    }
    if (!kind.startsWith("delete") || kind.includes("Forward")) return { from: at, to: at };
    const text = hands.text();
    let from = at;
    while (from > 0 && /\s/.test(text[from - 1] ?? "")) from--;
    return { from, to: at };
  };

  const spanOfRange = (range: AbstractRange): { from: number; to: number } | null => {
    const text = hands.text();
    const same = kept && range.collapsed && range.startContainer === kept.node;
    if (same && kept && range.startOffset === kept.offset) return { from: kept.at, to: kept.at };
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
  let whole = false;
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
    if (page.documentElement.classList.contains("untitled")) whole = true;
    const from = bytesAt(now, one.from);
    const added = BYTES.encode(one.insert).length;
    const by = added - BYTES.encode(was.slice(one.from, one.to)).length;
    laying = laying.then(async () => {
      unsaid = null;
      clearTimeout(resting);
      if (whole) return hands.redraw(hands.text());
      const near = await hands.near(now, from, from + added).catch(() => null);
      const sheet = page.querySelector(".page");
      const laid = sheet && near && patched(sheet, near, by);
      if (!sheet || !laid) {
        whole = true;
        return hands.redraw(hands.text());
      }
      hands.dressed(sheet, laid);
      rested(now);
      if (hands.text() === now) drawn();
    });
  };

  const put = (one: Edit, caret = one.from + one.insert.length, drawnWhole = false) => {
    const text = hands.text();
    const next = applied(text, one);
    if (next === text) return;
    undone.push({ text, at: one.from });
    redone.length = 0;
    kept = { node: page.body, offset: -1, at: caret };
    hands.write(next);
    if (drawnWhole) whole = true;
    laidOut(text, next, one);
  };

  const turned = (from: typeof undone, to: typeof undone) => {
    const back = from.pop();
    if (!back) return;
    to.push({ text: hands.text(), at: back.at });
    kept = { node: page.body, offset: -1, at: back.at };
    hands.write(back.text);
    whole = true;
    hands.redraw(back.text);
  };

  const inCode = (node: Node) =>
    Boolean(
      (node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement)?.closest(
        "code",
      ),
    );

  const pasted = (one: { from: number; to: number }, data: DataTransfer | null) => {
    const html = data?.getData("text/html") || null;
    const plain = data?.getData("text/plain") ?? "";
    hands
      .translated(html, plain)
      .then((said) => put({ ...one, insert: said }))
      .catch(() => put({ ...one, insert: plain }));
  };

  // A field laid over the page or inside it, as a link's address or a table's cell, writes its own words.
  const onSheet = (event: Event) => {
    const target = event.target as Element | null;
    if (target?.closest?.(FIELDS)) return false;
    return Boolean(page.querySelector(".page")?.contains(target));
  };

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
    if (!onSheet(event) || event.isComposing || event.inputType === "insertCompositionText") return;
    event.preventDefault();
    const kind = event.inputType;
    if (kind === "historyUndo") return turned(undone, redone);
    if (kind === "historyRedo") return turned(redone, undone);
    const broken = pastBreak(kind);
    if (broken) return put({ ...broken, insert: "" }, broken.from);
    const [target] = event.getTargetRanges?.() ?? [];
    const range = target ?? chosen();
    const whole = range && !range.collapsed;
    // A caret dropped on a table, a picture or a widget has no words under it to write among.
    if (range && !whole && unwritable(range.startContainer)) return;
    const one = (!whole && waited(kind)) || (range && (unbegun(kind, range) ?? spanOfRange(range)));
    if (!range || !one) return;
    const text = hands.text();
    const typed = event.data ?? event.dataTransfer?.getData("text/plain") ?? "";
    const begun = kept?.lead !== undefined && ghostly(range.startContainer) ? kept : null;
    if (begun && kind !== "insertText" && kind !== "insertReplacementText") return;
    if (begun) return firstKey(begun, typed);
    if (kind === "insertText" && typed === "/" && one.from === one.to && blank(text, one.from))
      return hands.blocks(caretPoint(), true);
    if (kind === "insertText" || kind === "insertReplacementText")
      return put({ ...one, insert: typed });
    if (kind === "insertParagraph")
      return put({ ...one, insert: parted(text, one.from, inCode(range.startContainer)) });
    if (kind === "insertLineBreak") return put({ ...one, insert: "\n" });
    if (kind === "insertFromPaste") return pasted(one, event.dataTransfer);
    if (kind.startsWith("delete")) return put({ ...one, insert: "" }, one.from);
    const pair = PAIRS[kind];
    if (pair) format(pair, pair);
  };

  let lastChosen: { from: number; to: number } | null = null;
  const noted = () => {
    besideLine();
    const range = chosen();
    const sheet = page.querySelector(".page");
    if (range && !range.collapsed && sheet?.contains(range.startContainer))
      lastChosen = spanOfRange(range);
  };

  const chosenNow = () => {
    const range = chosen();
    const one = range && !range.collapsed ? spanOfRange(range) : lastChosen;
    return one && one.from < one.to ? one : null;
  };

  // A format already around the words is taken off, as a second press of a button undoes the first.
  const worn = (one: { from: number; to: number }, open: string, close: string): Edit => {
    const text = hands.text();
    const inner = text.slice(one.from, one.to);
    const around =
      text.slice(one.from - open.length, one.from) === open &&
      text.slice(one.to, one.to + close.length) === close;
    return around
      ? { from: one.from - open.length, to: one.to + close.length, insert: inner }
      : { ...one, insert: `${open}${inner}${close}` };
  };

  const linked = (one: { from: number; to: number }) => {
    const held = page.getSelection();
    const box = held?.rangeCount ? held.getRangeAt(0).getBoundingClientRect() : null;
    const near = box ?? page.querySelector(".page")?.getBoundingClientRect();
    if (!near) return;
    void fieldAsked(page, near, t("linkAddress"), { kind: "url" }).then((to) => {
      if (!to) return;
      const inner = hands.text().slice(one.from, one.to);
      put({ ...one, insert: `[${inner}](${to})` });
    });
  };

  const format = (open: string, close: string) => {
    if (unformattable(page)) return;
    const one = chosenNow();
    if (!one) return;
    if (close === "]()") return linked(one);
    const change = worn(one, open, close);
    put(change, change.from + change.insert.length);
  };

  const started = (event: CompositionEvent) => {
    if (!onSheet(event)) return;
    const range = chosen();
    composing = range && spanOfRange(range);
  };
  const ended = (event: CompositionEvent) => {
    if (!onSheet(event)) return;
    const one = composing;
    composing = null;
    if (kept?.lead !== undefined && one?.from === kept.at) return firstKey(kept, event.data ?? "");
    if (one) put({ ...one, insert: event.data ?? "" });
  };

  // The sheet is written on whole, but a table, a drawing or a rule is changed by its own hands.
  // A task's labels are filled under, not renamed, until its sections are asked for; then only they are written.
  const opened = () => {
    const sheet = page.querySelector<HTMLElement>(".page");
    if (!sheet) return;
    sheet.setAttribute("contenteditable", "true");
    sheet.spellcheck = true;
    const look = page.documentElement.classList;
    const task = look.contains("untitled");
    const shaping = task && look.contains("shaping");
    for (const one of sheet.children) {
      const label = /^H[1-6]$/.test(one.tagName);
      const free = shaping ? label : one.matches(WRITABLE) && !(task && label);
      if (free) one.removeAttribute("contenteditable");
      else one.setAttribute("contenteditable", "false");
    }
    for (const one of sheet.querySelectorAll(".drop-section, .doc-card"))
      one.setAttribute("contenteditable", "false");
  };

  const caretLaid = () => {
    if (!kept) return;
    const unwritten = kept.lead !== undefined || !spotOf(page, hands.text(), kept.at);
    const ghost = unwritten ? ghostAt(page, hands.text(), kept.at) : null;
    const spot: [Node, number] | null = ghost ? [ghost, 0] : spotOf(page, hands.text(), kept.at);
    if (!spot) return;
    const held = page.getSelection();
    held?.collapse(spot[0], spot[1]);
    kept = { ...kept, node: spot[0], offset: spot[1] };
  };

  const drawn = () => {
    whole = false;
    opened();
    caretLaid();
  };

  // Colours, icons and cards land after a block is drawn and take the place of its words, the caret with them.
  const Watching = page.defaultView?.MutationObserver;
  const watching =
    Watching &&
    new Watching((records) => {
      if (records.some((one) => one.type === "attributes")) opened();
      if (kept && kept.offset >= 0 && !kept.node.isConnected) caretLaid();
    });
  watching?.observe(page.body, { childList: true, subtree: true });
  watching?.observe(page.documentElement, { attributes: true, attributeFilter: ["class"] });

  // A line begun from a hint, a label or a button takes what it opens with and its blank lines with its first key.
  const firstKey = (begun: Kept, typed: string) => {
    const lead = begun.lead ?? "";
    put(
      { from: begun.at, to: begun.at, insert: `${lead}${typed}${begun.tail ?? ""}` },
      begun.at + lead.length + typed.length,
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
    kept = { node: ghost, offset: 0, at, lead, tail };
    page.querySelector<HTMLElement>(".page")?.focus();
    page.getSelection()?.collapse(ghost, 0);
  };

  const begin = (at: number, seed: string, near: Element) => {
    const text = hands.text();
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
    if ((!box || (box.width === 0 && box.height === 0)) && ghost)
      box = ghost.getBoundingClientRect();
    const frame = page.defaultView?.frameElement?.getBoundingClientRect();
    return {
      x: (frame?.left ?? 0) + (box?.left ?? 0),
      y: (frame?.top ?? 0) + (box?.bottom ?? 0) + 4,
    };
  };

  const caretSpan = () => {
    const range = chosen();
    if (!range) return kept && kept.offset < 0 ? { from: kept.at, to: kept.at } : null;
    return unbegun("insertText", range) ?? spanOfRange(range);
  };

  const blockEnd = (at: number) => {
    const held = page.getSelection();
    const node = held?.anchorNode ?? null;
    const element = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
    const top = element?.closest(".page > [data-at]");
    const span = spanned(top?.getAttribute("data-at") ?? null);
    return span ? charsAt(hands.text(), span[1]) : at;
  };

  const slashed = (one: Slashed, drawnWhole = false): Edit | null => {
    const span = caretSpan();
    if (!span) return null;
    const text = hands.text();
    const end = Math.max(span.to, blockEnd(span.to));
    const how = placed(text, span.from, end, one);
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

  const besideLine = () => {
    const held = page.getSelection();
    const range = held?.rangeCount ? held.getRangeAt(0) : null;
    const sheet = page.querySelector<HTMLElement>(".page");
    if (!range || !sheet?.contains(range.startContainer) || !roomy(page)) {
      plus.hidden = true;
      return;
    }
    const ghost = page.querySelector(GHOST);
    let box = range.getBoundingClientRect();
    if (box.width === 0 && box.height === 0) {
      const node = range.startContainer;
      const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
      box = (ghost ?? element)?.getBoundingClientRect() ?? box;
    }
    const view = page.defaultView;
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const edge = sheet.getBoundingClientRect().left;
    plus.hidden = false;
    plus.style.top = `${(box.top + (view?.scrollY ?? 0)) / zoom}px`;
    plus.style.left = `${(edge + (view?.scrollX ?? 0)) / zoom + 16}px`;
  };

  // The caret goes with the words it stood in, as far into them as it was.
  // Words changed from beside the page, as a table's cell, are laid block by block and leave the caret where it is.
  const rewrite = (next: string) => {
    const text = hands.text();
    if (next === text) return;
    const one = changeOf(text, next);
    undone.push({ text, at: one.from });
    redone.length = 0;
    kept = null;
    hands.write(next);
    laidOut(text, next, one);
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

  const keyed = (event: KeyboardEvent) => {
    if (!onSheet(event)) return;
    if (event.altKey && (event.key === "ArrowUp" || event.key === "ArrowDown"))
      return stepped(event);
    if (event.key !== "/" || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    hands.blocks(caretPoint(), false);
  };

  const ungrip = gripped(page, { moved: movedOnto, asked: (top, at) => hands.acts(at, top) });

  // The page's events come from its own window, so a key is told by what it carries, not by its class.
  const moved = (event: Event) => {
    const key = "key" in event ? String(event.key) : null;
    if (key !== null && !key.startsWith("Arrow") && !/^(Home|End|Page)/.test(key)) return;
    kept = null;
  };

  page.addEventListener("beforeinput", asked);
  page.addEventListener("compositionstart", started);
  page.addEventListener("compositionend", ended);
  page.addEventListener("mousedown", moved);
  page.addEventListener("click", quoted);
  page.addEventListener("keydown", moved);
  page.addEventListener("selectionchange", noted);
  page.addEventListener("keydown", keyed);
  opened();
  return {
    drawn,
    format,
    formattable: () => !unformattable(page),
    rewrite,
    chosen: () => {
      const one = chosenNow();
      return one ? hands.text().slice(one.from, one.to) : null;
    },
    slashed,
    unitActed,
    begin,
    hintWords,
    typed: (said) => {
      const span = caretSpan();
      if (span) put({ ...span, insert: said });
    },
    cut: () => {
      const one = chosenNow();
      if (one) put({ ...one, insert: "" }, one.from);
    },
    stop: () => {
      told();
      watching?.disconnect();
      page.removeEventListener("selectionchange", noted);
      page.removeEventListener("keydown", keyed);
      plus.remove();
      ungrip();
      page.removeEventListener("mousedown", moved);
      page.removeEventListener("click", quoted);
      page.removeEventListener("keydown", moved);
      page.removeEventListener("beforeinput", asked);
      page.removeEventListener("compositionstart", started);
      page.removeEventListener("compositionend", ended);
      page.querySelector<HTMLElement>(".page")?.removeAttribute("contenteditable");
    },
  };
};
