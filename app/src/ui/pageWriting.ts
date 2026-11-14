import { bytesAt } from "tisty-editor/read";
import { t } from "../locales";
import type { PlacedNear } from "../placing";
import { addressAsked } from "./pageLinking";
import { patched } from "./pagePatching";
import { GHOST, ghostAt, sourceAt, spotOf } from "./pageSpots";

type Edit = { readonly from: number; readonly to: number; readonly insert: string };
type Kept = { readonly node: Node; readonly offset: number; readonly at: number };

export type Hands = {
  readonly text: () => string;
  readonly write: (next: string) => void;
  readonly shown: (now: string) => void;
  readonly redraw: (now: string) => void;
  readonly near: (body: string, from: number, to: number) => Promise<PlacedNear>;
  readonly dressed: (sheet: Element, laid: readonly Element[]) => void;
  readonly translated: (html: string | null, plain: string) => Promise<string>;
};

const BYTES = new TextEncoder();
// What the rest of the window reads of the page catches up once the keys rest, not on each one.
const RESTED_MS = 250;

export type PageWriting = {
  readonly drawn: () => void;
  readonly stop: () => void;
  readonly format: (open: string, close: string) => void;
  readonly chosen: () => string | null;
  readonly cut: () => void;
};

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

  // Out of a paragraph not yet written, going back leaves it and joins the block before.
  const unbegun = (kind: string, range: AbstractRange): { from: number; to: number } | null => {
    if (!kept || !range.collapsed || !ghostly(range.startContainer)) return null;
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

  const put = (one: Edit, caret = one.from + one.insert.length) => {
    const text = hands.text();
    const next = applied(text, one);
    if (next === text) return;
    undone.push({ text, at: one.from });
    redone.length = 0;
    kept = { node: page.body, offset: -1, at: caret };
    hands.write(next);
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

  // A field laid over the page, as a link's address, writes its own words.
  const onSheet = (event: Event) =>
    Boolean(page.querySelector(".page")?.contains(event.target as Node | null));

  const asked = (event: InputEvent) => {
    if (!onSheet(event) || event.isComposing || event.inputType === "insertCompositionText") return;
    event.preventDefault();
    const kind = event.inputType;
    if (kind === "historyUndo") return turned(undone, redone);
    if (kind === "historyRedo") return turned(redone, undone);
    const [target] = event.getTargetRanges?.() ?? [];
    const range = target ?? chosen();
    const whole = range && !range.collapsed;
    const one = (!whole && waited(kind)) || (range && (unbegun(kind, range) ?? spanOfRange(range)));
    if (!range || !one) return;
    const text = hands.text();
    if (kind === "insertText" || kind === "insertReplacementText")
      return put({ ...one, insert: event.data ?? event.dataTransfer?.getData("text/plain") ?? "" });
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
    void addressAsked(page, near, t("linkAddress")).then((to) => {
      if (!to) return;
      const inner = hands.text().slice(one.from, one.to);
      put({ ...one, insert: `[${inner}](${to})` });
    });
  };

  const format = (open: string, close: string) => {
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
    if (one) put({ ...one, insert: event.data ?? "" });
  };

  // The sheet is written on whole, but a table, a drawing or a rule is changed by its own hands.
  const opened = () => {
    const sheet = page.querySelector<HTMLElement>(".page");
    if (!sheet) return;
    sheet.setAttribute("contenteditable", "true");
    sheet.spellcheck = true;
    for (const one of sheet.children) {
      if (!one.matches(WRITABLE)) one.setAttribute("contenteditable", "false");
    }
  };

  const drawn = () => {
    whole = false;
    opened();
    if (!kept) return;
    const ghost = spotOf(page, hands.text(), kept.at) ? null : ghostAt(page, hands.text(), kept.at);
    const spot: [Node, number] | null = ghost ? [ghost, 0] : spotOf(page, hands.text(), kept.at);
    if (!spot) return;
    const held = page.getSelection();
    held?.collapse(spot[0], spot[1]);
    kept = { node: spot[0], offset: spot[1], at: kept.at };
  };

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
  page.addEventListener("keydown", moved);
  page.addEventListener("selectionchange", noted);
  opened();
  return {
    drawn,
    format,
    chosen: () => {
      const one = chosenNow();
      return one ? hands.text().slice(one.from, one.to) : null;
    },
    cut: () => {
      const one = chosenNow();
      if (one) put({ ...one, insert: "" }, one.from);
    },
    stop: () => {
      told();
      page.removeEventListener("selectionchange", noted);
      page.removeEventListener("mousedown", moved);
      page.removeEventListener("keydown", moved);
      page.removeEventListener("beforeinput", asked);
      page.removeEventListener("compositionstart", started);
      page.removeEventListener("compositionend", ended);
      page.querySelector<HTMLElement>(".page")?.removeAttribute("contenteditable");
    },
  };
};
