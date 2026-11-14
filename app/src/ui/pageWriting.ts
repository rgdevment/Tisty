import { type Asking, SLASHED, type Slashed } from "tisty-editor/edit";
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
import { endIn, endOf, GHOST, ghostAt, sourceAt, spotOf } from "./pageSpots";
import { tagList, tagMatch } from "./pageTags";
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

type Turn = { readonly text: string; readonly at: number; readonly after?: number };

/** What Ctrl+Z walks back through, kept by whoever shows the page so a turn to the Markdown keeps it. */
export type Turns = { readonly undone: Turn[]; readonly redone: Turn[]; left?: string };

export const turnsAnew = (): Turns => ({ undone: [], redone: [] });

export type Hands = {
  readonly turns?: Turns;
  /** A block, the window or the page was left: what was written there is told once, never each key. */
  readonly settle?: () => void;
  readonly text: () => string;
  readonly write: (next: string) => void;
  readonly shown: (now: string) => void;
  readonly redraw: (now: string) => void;
  readonly near: (body: string, from: number, to: number) => Promise<PlacedNear>;
  readonly dressed: (sheet: Element, laid: readonly Element[]) => void;
  readonly translated: (html: string | null, plain: string) => Promise<string>;
  readonly ask: Asking["ask"];
  readonly blocks: (at: Point, slash: boolean) => void;
  readonly acts: (at: { x: number; y: number }, top: Element) => void;
  readonly tags?: () => readonly string[];
  readonly pasted?: (files: readonly File[]) => Promise<string>;
};

type Point = { readonly x: number; readonly y: number; readonly top?: number };

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
  readonly chosen: () => string | null;
  readonly cut: () => void;
  readonly slashed: (one: Slashed, drawnWhole?: boolean) => Edit | null;
  readonly typed: (said: string) => void;
  readonly unitActed: (top: Element, act: Act) => void;
  readonly begin: (at: number, seed: string, near: Element) => void;
  readonly hintWords: (at: [number, number], hint: HTMLElement) => void;
};

const FIELDS = "input, textarea, select";

const LITERAL = "pre, code, kbd, figure.code > figcaption";

const WRITABLE =
  ".page > :is(p, h1, h2, h3, h4, h5, h6, ul, ol, blockquote, figure.code, .footnote, details)";
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
  const have = (/(?:\r?\n)*$/.exec(before)?.[0].match(/\n/g) ?? []).length;
  return "\n".repeat(Math.max(0, (joins ? 1 : 2) - have));
};

// An item begun past the blank line that ends a list goes back to the list, so the list stays tight.
const joinedAt = (text: string, at: number, seed: string) => {
  const blank = /(?:\r?\n){2,}$/.exec(text.slice(0, at))?.[0];
  if (!blank || !LIST_ITEM.test(seed)) return at;
  const lines = text.slice(0, at - blank.length).split("\n");
  if (!LIST_ITEM.test(lines[lines.length - 1] ?? "")) return at;
  return at - blank.length + (blank.startsWith("\r") ? 2 : 1);
};

const closing = (text: string, at: number) => {
  const after = text.slice(at);
  if (!after.trim()) return "";
  const have = (/^(?:\r?\n)*/.exec(after)?.[0].match(/\n/g) ?? []).length;
  return "\n".repeat(Math.max(0, 2 - have));
};

const MARK = /\*\*|__|\+\+|~~|==|\*|_|`|\[|\]/g;
const LINK_END = /^\]\([^)\s]*(?:\s+"[^"]*")?\)/;

// Marks right after the caret close the words before it only when the line opened them, never a next word's.
const closedTokens = (text: string, at: number): string[] => {
  const line = text.slice(text.lastIndexOf("\n", at - 1) + 1, at);
  const seen = new Map<string, number>();
  const count = (kind: string) => seen.get(kind) ?? 0;
  for (const [token] of line.matchAll(MARK)) seen.set(token, count(token) + 1);
  const found: string[] = [];
  let end = at;
  for (;;) {
    const link = LINK_END.exec(text.slice(end))?.[0];
    if (link && count("[") > count("]")) {
      seen.set("]", count("]") + 1);
      found.push(link);
      end += link.length;
      continue;
    }
    const stars = /^(?:\*+|_+)/.exec(text.slice(end))?.[0];
    const star = stars?.[0] ?? "";
    const twice = star + star;
    const pick =
      stars && stars.length > 1 && count(twice) % 2 ? twice : count(star) % 2 ? star : "";
    CLOSER.lastIndex = end;
    const token = stars ? pick : CLOSER.exec(text)?.[0];
    const kind = token?.startsWith("==") ? "==" : token;
    if (!token || !kind || count(kind) % 2 === 0) return found;
    seen.set(kind, count(kind) + 1);
    found.push(token);
    end += token.length;
  }
};

const closedRun = (text: string, at: number) => at + closedTokens(text, at).join("").length;

const pastClose = (text: string, at: number): number | null => {
  if (!/\S/.test(text[at - 1] ?? "")) return null;
  const line = text.slice(text.lastIndexOf("\n", at - 1) + 1, at);
  if ((line.match(/`/g)?.length ?? 0) % 2) return null;
  const end = closedRun(text, at);
  return end > at ? end : null;
};

// Stars and underscores stack, so a run of three is bold and italic at once and two alone are only bold.
const starred = (text: string, one: { from: number; to: number }, open: string) => {
  const mark = open[0] ?? "*";
  let before = 0;
  while (text[one.from - before - 1] === mark) before++;
  let after = 0;
  while (text[one.to + after] === mark) after++;
  const most = Math.min(before, after);
  return open.length === 1 ? most % 2 === 1 : most >= 2;
};

const CLOSER = /\*\*|__|\+\+|~~|==(?:\{[a-z]+\})?|\*|_|`/y;

type Marks = readonly (readonly [string, string])[];

const starsBefore = (text: string, at: number, star: string) => {
  let many = 0;
  while (text[at - many - 1] === star) many++;
  return many;
};

// A format closed just before the caret is entered again rather than opened anew, so no marks stand back to back.
const rejoined = (text: string, at: number, marks: Marks): [number, Marks] => {
  let from = at;
  let left = marks;
  for (;;) {
    const found = left.findIndex(([, close]) => {
      const star = /^(\*|_)\1?$/.test(close) ? (close[0] ?? "") : "";
      const run = star ? starsBefore(text, from, star) : 0;
      const shut = star
        ? close.length === 1
          ? run % 2 === 1
          : run >= 2
        : text.slice(from - close.length, from) === close;
      const edge = text[from - (star ? run : close.length) - 1] ?? "";
      return shut && /\S/.test(edge);
    });
    if (found < 0) return [from, left];
    from -= left[found]?.[1].length ?? 0;
    left = left.filter((_, index) => index !== found);
  }
};

const PAIRED = ["**", "__", "++", "~~", "==", "*", "_", "`"];

const unpaired = (text: string, one: { from: number; to: number }) => {
  let { from, to } = one;
  if (from === to) return one;
  for (;;) {
    const mark = PAIRED.find(
      (one) => text.slice(from - one.length, from) === one && text.startsWith(one, to),
    );
    if (!mark) return { from, to };
    const close = mark === "==" ? (/^==(?:\{[a-z]+\})?/.exec(text.slice(to))?.[0] ?? mark) : mark;
    from -= mark.length;
    to += close.length;
  }
};

const unsplit = (text: string, one: { from: number; to: number }) => ({
  from: text[one.from - 1] === "\r" && text[one.from] === "\n" ? one.from - 1 : one.from,
  to: text[one.to - 1] === "\r" && text[one.to] === "\n" ? one.to + 1 : one.to,
});

// A body written with CRLF keeps it, so a key never leaves a bare line feed among its lines.
const linedAs = (text: string, one: Edit, caret: number): [Edit, number] => {
  if (!text.includes("\r\n") || !/(?<!\r)\n/.test(one.insert)) return [one, caret];
  const insert = one.insert.replace(/\r?\n/g, "\r\n");
  const within = Math.max(0, Math.min(caret - one.from, one.insert.length));
  const added = one.insert.slice(0, within).match(/(?<!\r)\n/g)?.length ?? 0;
  const past = caret - one.from > one.insert.length ? insert.length - one.insert.length : added;
  return [{ ...one, insert }, caret + past];
};

const trimmed = (text: string, one: { from: number; to: number } | null) => {
  if (!one) return null;
  let { from, to } = one;
  while (from < to && /\s/.test(text[from] ?? "")) from++;
  while (to > from && /\s/.test(text[to - 1] ?? "")) to--;
  return from < to ? { from, to } : null;
};

const ITEM_LEAD = /^(\s*)((?:[-*+]|\d+[.)])\s+)/;

// An empty item nested in another goes up to its parent's level instead of leaving the whole list.
const outdented = (text: string, at: number): { edit: Edit; caret: number } | null => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const ends = text.indexOf("\n", at);
  const line = text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "");
  const lead = ITEM_LEAD.exec(line)?.[1].length ?? 0;
  if (!lead || at < start + line.length || !EMPTY_ITEM.test(line)) return null;
  const above = text.slice(0, start).split("\n").reverse();
  const parent = above.find(
    (one) => ITEM_LEAD.test(one) && (ITEM_LEAD.exec(one)?.[1].length ?? 0) < lead,
  );
  const to = parent ? (ITEM_LEAD.exec(parent)?.[1].length ?? 0) : 0;
  return {
    edit: { from: start, to: start + lead, insert: " ".repeat(to) },
    caret: start + to + line.length - lead,
  };
};

const EMPTY_ITEM = /^\s*(?:>\s?)*\s*(?:[-*+]|\d+[.)])(?:\s+(?:\[[ xX]\]\s*)?)?$/;
const EMPTY_QUOTE = /^\s*(?:>\s?)+$/;

// Enter on an item or a quote line with nothing written leaves the list or the quote, taking that line away.
const leftBehind = (text: string, at: number): { from: number; to: number } | null => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const ends = text.indexOf("\n", at);
  const line = text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "");
  if (at < start + line.length || !(EMPTY_ITEM.test(line) || EMPTY_QUOTE.test(line))) return null;
  const above = text.slice(0, Math.max(0, start - 1));
  const before = above.lastIndexOf("\n") + 1;
  const bare = EMPTY_QUOTE.test(line) && EMPTY_QUOTE.test(above.slice(before).replace(/\r$/, ""));
  return { from: bare ? before : start, to: start + line.length };
};

// Words partly in a format take it whole, so the marks inside them go before the new pair goes around.
const bareOf = (inner: string, open: string) => {
  if (open === "*") return inner.replace(/(?<!\*)\*(?!\*)/g, "");
  if (open === "_") return inner.replace(/(?<!_)_(?!_)/g, "");
  if (open.startsWith("==")) return inner.replace(/==(?:\{[a-z]+\})?/g, "");
  return inner.split(open).join("");
};

// A choice that starts or ends inside a format takes that format whole, so no mark is left without its partner.
const MARK_CHAR = /[*_~+=`]/;

const covered = (text: string, cut: { from: number; to: number }, open: string) => {
  const one = { ...cut };
  while (
    one.from > 0 &&
    MARK_CHAR.test(text[one.from - 1] ?? "") &&
    MARK_CHAR.test(text[one.from] ?? "")
  )
    one.from--;
  while (
    one.to < text.length &&
    MARK_CHAR.test(text[one.to - 1] ?? "") &&
    MARK_CHAR.test(text[one.to] ?? "")
  )
    one.to++;
  const mark = open.startsWith("==") ? "==" : open;
  const single = mark === "*" || mark === "_";
  const lone = single ? new RegExp(`(?<!\\${mark})\\${mark}(?!\\${mark})`, "g") : null;
  const count = (said: string) =>
    lone ? (said.match(lone)?.length ?? 0) : said.split(mark).length - 1;
  if (!mark || count(text.slice(one.from, one.to)) % 2 === 0) return one;
  const start = text.lastIndexOf("\n", one.from - 1) + 1;
  if (count(text.slice(start, one.from)) % 2 === 1) {
    const before = text.slice(start, one.from);
    const at = lone ? [...before.matchAll(lone)].pop()?.index : before.lastIndexOf(mark);
    return at === undefined || at < 0 ? one : { from: start + at, to: one.to };
  }
  const after = text.slice(one.to);
  const at = lone ? after.search(lone) : after.indexOf(mark);
  return at < 0 ? one : { from: one.from, to: one.to + at + mark.length };
};

const applied = (text: string, one: Edit) =>
  `${text.slice(0, one.from)}${one.insert}${text.slice(one.to)}`;

export const writtenOn = (page: Document, hands: Hands): PageWriting => {
  let kept: Kept | null = null;
  let composing: { from: number; to: number } | null = null;
  let pending: {
    readonly at: number;
    readonly marks: readonly (readonly [string, string])[];
  } | null = null;
  let carried: { readonly from: number; readonly close: string; readonly caret: number } | null =
    null;
  const openOf = (wait: NonNullable<typeof pending>) => wait.marks.map(([open]) => open).join("");
  const closeOf = (wait: NonNullable<typeof pending>) =>
    wait.marks
      .map(([, close]) => close)
      .reverse()
      .join("");
  let typing = false;
  const turns = hands.turns ?? turnsAnew();
  const { undone, redone } = turns;
  // What was written in the Markdown since the page was left is one step back.
  if (turns.left !== undefined && turns.left !== hands.text()) {
    undone.push({ text: turns.left, at: changeOf(turns.left, hands.text()).from });
    redone.length = 0;
  }
  turns.left = undefined;

  // Until the page is drawn again it shows the words before the last key, so the caret kept in the words decides.
  const waited = (kind: string): { from: number; to: number } | null => {
    if (!kept || kept.lead !== undefined) return null;
    if (kept.offset >= 0) {
      const caret = chosen();
      const here =
        caret?.collapsed && caret.startContainer === kept.node && caret.startOffset === kept.offset;
      if (!here || (kind !== "deleteContentBackward" && kind !== "deleteContentForward"))
        return null;
    }
    const { at } = kept;
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
    if (kept) kept = { ...kept, node: page.body, offset: -1 };
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
    const unlaid = kept?.lead !== undefined && kept.offset < 0;
    if (!kept || !range.collapsed || !(unlaid || ghostly(range.startContainer))) return null;
    const { at } = kept;
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

  const put = (asked: Edit, at = asked.from + asked.insert.length, drawnWhole = false) => {
    const text = hands.text();
    const [one, caret] = linedAs(text, asked, at);
    const next = applied(text, one);
    if (next === text) return;
    const stood = kept && kept.offset < 0 ? kept.at : (caretSpan()?.from ?? one.from);
    undone.push({ text, at: Math.min(stood, text.length), after: caret });
    redone.length = 0;
    kept = { node: page.body, offset: -1, at: caret };
    hands.write(next);
    if (drawnWhole) whole = true;
    laidOut(text, next, one);
  };

  const turned = (from: typeof undone, to: typeof undone, forward = false) => {
    const back = from.pop();
    if (!back) return;
    pending = null;
    to.push({ text: hands.text(), at: back.at, after: back.after });
    kept = { node: page.body, offset: -1, at: forward ? (back.after ?? back.at) : back.at };
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

  // A slash opens the blocks where a word could start, as it always did; inside code it is only a slash.
  const slashes = (text: string, one: { from: number; to: number }, range: AbstractRange) =>
    one.from === one.to &&
    !inCode(range.startContainer) &&
    (blank(text, one.from) || /(^|\s)$/.test(text.slice(0, one.from).split("\n").pop() ?? ""));

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
    if (kind === "insertParagraph") hands.settle?.();
    typing = kind === "insertText" || kind.startsWith("delete");
    if (kind !== "insertText" && !kind.startsWith("format")) pending = null;
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
    const whole = Boolean(picked && !picked.collapsed);
    // A caret dropped on a table, a picture or a widget has no words under it to write among.
    if (range && !whole && unwritable(range.startContainer)) return;
    const held = picked ?? range;
    const begunNow =
      kept?.lead !== undefined &&
      (kept.offset < 0 || Boolean(held && ghostly(held.startContainer)));
    if (begunNow && kind.startsWith("delete")) return dropBegun();
    const one =
      (!whole && waited(kind)) || (range && ((held && unbegun(kind, held)) ?? spanOfRange(range)));
    if (!range || !one) return;
    const text = hands.text();
    const typed = event.data ?? event.dataTransfer?.getData("text/plain") ?? "";
    const begun = begunNow ? kept : null;
    if (begun && kind !== "insertText" && kind !== "insertReplacementText") return;
    if (begun && kind === "insertText" && typed === "/") return hands.blocks(caretPoint(), true);
    if (begun) return firstKey(begun, typed);
    if (kind === "insertText" && typed === "/" && slashes(text, one, range))
      return hands.blocks(caretPoint(), true);
    if (kind === "insertText" && one.from === one.to) {
      if (pending?.at === one.from) return pendingKey(one.from, typed);
      const opened = text.slice(text.lastIndexOf("\n", one.from - 1) + 1, one.from);
      if (typed === " " && /^\s*(?:>\s?)*\[ ?\]$/.test(opened)) {
        const box = `${opened.replace(/\[ ?\]$/, "")}- [ ] `;
        return put({ from: one.from - opened.length, to: one.from, insert: box });
      }
      const carry = carried?.caret === one.from ? carried : null;
      carried = null;
      if (carry && !typed.trim()) {
        carried = { ...carry, caret: one.from + typed.length };
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
        carried = { from: one.from, close: text.slice(one.from, past), caret: past + typed.length };
        return put({ from: past, to: past, insert: typed });
      }
    }
    carried = null;
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
        kept = {
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

  let lastChosen: { from: number; to: number } | null = null;
  // A piece drawn whole, as code or a tag, takes no keys, so a caret dropped inside it stands right after it.
  const outOfSolid = (range: AbstractRange) => {
    const node = range.startContainer;
    const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
    const atom = element?.closest('.page [data-src][contenteditable="false"]');
    if (!atom?.parentElement || atom.parentElement.closest('[contenteditable="false"]')) return;
    const [holder, offset] = endOf(atom);
    page.getSelection()?.collapse(holder, offset);
  };

  const noted = () => {
    besideLine();
    const range = chosen();
    const sheet = page.querySelector(".page");
    if (range && !range.collapsed && sheet?.contains(range.startContainer))
      lastChosen = spanOfRange(range);
    else if (range?.collapsed) {
      lastChosen = null;
      outOfSolid(range);
    }
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
    const around = /^(\*|_)\1?$/.test(open)
      ? starred(text, one, open)
      : text.slice(one.from - open.length, one.from) === open &&
        text.slice(one.to, one.to + close.length) === close;
    return around
      ? { from: one.from - open.length, to: one.to + close.length, insert: inner }
      : { ...one, insert: `${open}${bareOf(inner, open)}${close}` };
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
    const carry = carried;
    carried = null;
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
    pending = null;
    const change = worn(one, open, close);
    put(change, change.from + change.insert.length);
  };

  const waitFor = (open: string, close: string) => {
    const at = caretSpan()?.from;
    if (at === undefined) return;
    const text = hands.text();
    const marks = pending?.at === at ? pending.marks : [];
    if (marks.some(([one]) => one === open)) {
      const rest = marks.filter(([one]) => one !== open);
      pending = rest.length ? { at, marks: rest } : null;
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
      kept = { node: page.body, offset: -1, at: past };
      pending = next.length ? { at: past, marks: next } : null;
      return;
    }
    pending = { at, marks: [...marks, [open, close]] };
    focused();
  };

  const pendingKey = (at: number, typed: string) => {
    const wait = pending;
    if (!wait) return;
    if (!typed.trim()) {
      put({ from: at, to: at, insert: typed });
      pending = { ...wait, at: at + typed.length };
      return;
    }
    pending = null;
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
    if (!onSheet(event)) return;
    const range = chosen();
    composing = range && spanOfRange(range);
  };
  const ended = (event: CompositionEvent) => {
    if (!onSheet(event)) return;
    const one = composing;
    composing = null;
    typing = true;
    if (kept?.lead !== undefined && one?.from === kept.at) return firstKey(kept, event.data ?? "");
    if (one && one.from === one.to && pending?.at === one.from)
      return pendingKey(one.from, event.data ?? "");
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
    // A step's box and its ↗ are no place for the caret, so ↑ and ↓ go from step to step.
    for (const one of sheet.querySelectorAll(".drop-section, .doc-card, .check, .to-part"))
      one.setAttribute("contenteditable", "false");
    // Code is spelled as the language wants it, so the system's speller has nothing to say there.
    for (const one of sheet.querySelectorAll(LITERAL)) one.setAttribute("spellcheck", "false");
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
    if (!kept) return;
    const unwritten = kept.lead !== undefined || !spotOf(page, hands.text(), kept.at);
    const ghost = unwritten ? ghostAt(page, hands.text(), kept.at) : null;
    const found: [Node, number] | null = ghost ? [ghost, 0] : spotOf(page, hands.text(), kept.at);
    if (!found) return;
    focused();
    const spot = summaryAt(found, kept.at) ?? found;
    // A block drawn empty, as a heading just begun, holds no line for the caret, so it gets one.
    const [holder] = spot;
    const bare = holder.nodeType === Node.ELEMENT_NODE ? (holder as Element) : null;
    if (bare && !bare.textContent && !bare.querySelector("br"))
      bare.append(page.createElement("br"));
    const held = page.getSelection();
    held?.collapse(spot[0], spot[1]);
    kept = { ...kept, node: spot[0], offset: spot[1] };
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
    whole = false;
    opened();
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
      if (records.some((one) => one.type === "attributes")) opened();
      if (kept && kept.offset >= 0 && !kept.node.isConnected) caretLaid();
    });
  watching?.observe(page.body, { childList: true, subtree: true });
  watching?.observe(page.documentElement, { attributes: true, attributeFilter: ["class"] });

  // A line begun from a hint, a label or a button takes what it opens with and its blank lines with its first key.
  const firstKey = (begun: Kept, typed: string) => {
    const lead = begun.lead ?? "";
    const wait = pending?.at === begun.at && typed.trim() ? pending : null;
    pending = null;
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
    kept = { node: ghost, offset: 0, at, lead, tail };
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

  // A caret on an empty line measures nothing, so the line's own element stands for it, never the whole sheet.
  const lineOfCaret = (range: Range | null | undefined): Element | null => {
    const node = range?.startContainer;
    if (!node) return null;
    const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
    const inside = element?.matches(".page") ? element.childNodes[range.startOffset] : null;
    const line = inside?.nodeType === Node.ELEMENT_NODE ? (inside as Element) : element;
    return line && !line.matches(".page") ? line : null;
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
    if (kept && kept.offset < 0) return { from: kept.at, to: kept.at };
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
    if (kept?.lead !== undefined && kept.at > 0 && hands.text()[kept.at - 1] !== "\n")
      firstKey(kept, "");
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
  const rewrite = (next: string) => {
    const text = hands.text();
    if (next === text) return;
    const one = changeOf(text, next);
    const held = chosen();
    const sheet = page.querySelector(".page");
    const was = held?.collapsed && sheet?.contains(held.startContainer) ? caretSpan()?.from : null;
    undone.push({ text, at: one.from });
    redone.length = 0;
    const by = one.insert.length - (one.to - one.from);
    kept =
      was === null || was === undefined
        ? null
        : { node: page.body, offset: -1, at: was <= one.from ? was : Math.max(one.from, was + by) };
    pending = null;
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
      kept && kept.offset < 0
        ? kept.at
        : Math.min(text.length, laid.from + laid.insert.replace(/\s+$/, "").length);
    while (at > 0 && /[\r\n]/.test(text[at - 1] ?? "")) at--;
    kept = {
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
    const text = hands.text();
    const start = text.lastIndexOf("\n", at - 1) + 1;
    const ends = text.indexOf("\n", at);
    const line = text.slice(start, ends < 0 ? text.length : ends);
    const own = ITEM_LEAD.exec(line);
    if (!own) return;
    const lead = own[1].length;
    if (event.shiftKey) {
      const above = text.slice(0, start).split("\n").reverse();
      const parent = above.find(
        (one) => ITEM_LEAD.test(one) && (ITEM_LEAD.exec(one)?.[1].length ?? 0) < lead,
      );
      const to = parent ? (ITEM_LEAD.exec(parent)?.[1].length ?? 0) : 0;
      if (to === lead) return;
      put(
        { from: start, to: start + lead, insert: " ".repeat(to) },
        Math.max(start, at - (lead - to)),
      );
      return;
    }
    const before =
      text
        .slice(0, Math.max(0, start - 1))
        .split("\n")
        .pop() ?? "";
    const prior = ITEM_LEAD.exec(before);
    if (!prior) return;
    const to = (prior[1].length ?? 0) + (prior[2].length ?? 0);
    if (to <= lead) return;
    put({ from: start, to: start + lead, insert: " ".repeat(to) }, at + (to - lead));
  };

  const blockOfCaret = () => {
    const node = chosen()?.startContainer;
    const element = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
    return element?.closest(".page [data-at]") ?? null;
  };

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
    if (!onSheet(event)) return;
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
      pending = null;
      carried = null;
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
    typing = false;
    const text = hands.text();
    const to = Math.min(match.to, text.length);
    put({ from: Math.min(match.from, to), to, insert: `#${tag}` });
  });

  const offerTags = () => {
    const at = kept?.at;
    const range = chosen();
    const known = hands.tags;
    const open = typing && at !== undefined && known && !(range && inCode(range.startContainer));
    tags.offer(open && known ? tagMatch(hands.text(), at, known()) : null);
  };

  const pastedFiles = (event: ClipboardEvent) => {
    const files = Array.from(event.clipboardData?.files ?? []);
    const attach = hands.pasted;
    if (!onSheet(event) || !files.length || !attach || event.clipboardData?.getData("text/plain"))
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
    if (key === null || /^(ArrowUp|ArrowDown|Page)/.test(key) || across) hands.settle?.();
    kept = null;
    pending = null;
    carried = null;
    typing = false;
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
  opened();
  return {
    drawn,
    format,
    formattable: () => !unformattable(page),
    caretAt: () => caretSpan()?.from ?? null,
    placeAt: (at: number) => {
      kept = { node: page.body, offset: -1, at: Math.min(at, hands.text().length) };
      page.querySelector<HTMLElement>(".page")?.focus({ preventScroll: true });
      caretLaid();
      const node = kept?.node;
      const holder = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;
      holder?.scrollIntoView?.({ block: "nearest" });
    },
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
      if (kept?.lead !== undefined) return firstKey(kept, said);
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
