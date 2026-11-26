import { copiedHtml } from "../core";

/** Tisty's own words exactly as written, so pasting from one page to another loses nothing. */
export const TISTY_MARKDOWN = "text/x-tisty-markdown";

/** Words copied from a page in each face another app may take. */
export interface Copied {
  tisty: string;
  html: string;
  plain: string;
}

const KEPT = new Set([
  "P",
  "H1",
  "H2",
  "H3",
  "H4",
  "H5",
  "H6",
  "STRONG",
  "B",
  "EM",
  "I",
  "U",
  "S",
  "DEL",
  "INS",
  "MARK",
  "CODE",
  "PRE",
  "A",
  "UL",
  "OL",
  "LI",
  "BLOCKQUOTE",
  "TABLE",
  "THEAD",
  "TBODY",
  "TFOOT",
  "TR",
  "TH",
  "TD",
  "BR",
  "HR",
  "DETAILS",
  "SUMMARY",
  "SUB",
  "SUP",
  "KBD",
]);

// Word and the browsers rebuild a formula from its MathML; its letters alone would read as other numbers.
const MATH = new Set(
  "math semantics mrow mi mo mn ms mtext mspace msup msub msubsup mfrac msqrt mroot mstyle mpadded mphantom menclose mtable mtr mtd munder mover munderover".split(
    " ",
  ),
);

// A reader without formatting still tells a power, an index and a fraction apart.
const MATH_SAID: Record<string, string[]> = {
  msup: ["^"],
  msub: ["_"],
  mfrac: ["/"],
  msubsup: ["_", "^"],
};

// The editor's own furniture, which the page carries but is no part of the words.
const FURNITURE = [
  "button",
  "svg",
  "input",
  "script",
  "style",
  "[data-ghost]",
  "[aria-hidden='true']",
  ".ico",
  ".page-drop",
  ".page-drop-caret",
  ".tag-list",
  ".diagram-bar",
  ".math-bar",
  ".picture-away",
  ".note-off",
  ".to-part",
].join(", ");

const LINKS = /^(https?:|mailto:)/i;

const SPANS = new Set(["colspan", "rowspan"]);

// Word and mail drop a colour with transparency, so each pen leaves as the solid tint it shows over white paper.
const PENS: Record<string, string> = {
  yellow: "#fdeeb4",
  red: "#fecaca",
  orange: "#fed7aa",
  amber: "#fde68a",
  green: "#c6f0d5",
  teal: "#c3f3ea",
  blue: "#ccdefd",
  indigo: "#c7d2fe",
  purple: "#e9d5ff",
  pink: "#fad3e6",
  brown: "#e7d3c3",
  gray: "#e5e7eb",
};

const BLOCKS = new Set([
  "P",
  "H1",
  "H2",
  "H3",
  "H4",
  "H5",
  "H6",
  "PRE",
  "UL",
  "OL",
  "BLOCKQUOTE",
  "TABLE",
  "DETAILS",
  "SUMMARY",
  "DIV",
  "FIGURE",
]);

// The letter the editor sows beside solid pieces to hold the caret, which no reader should get.
const UNSEEN = new RegExp(String.fromCharCode(0x200b), "g");

const ELEMENT = 1;
const TEXT = 3;

const kept = (from: Element, into: Element) => {
  const tag = from.tagName;
  if (tag === "A") into.setAttribute("href", from.getAttribute("href") ?? "");
  if (from.localName === "math" && from.hasAttribute("display"))
    into.setAttribute("display", from.getAttribute("display") ?? "");
  if (tag === "OL" && from.hasAttribute("start"))
    into.setAttribute("start", from.getAttribute("start") ?? "");
  for (const name of from.getAttributeNames())
    if ((tag === "TD" || tag === "TH") && SPANS.has(name))
      into.setAttribute(name, from.getAttribute(name) ?? "");
  if (tag === "MARK") {
    const tint = PENS[from.getAttribute("data-pen") ?? "yellow"] ?? PENS.yellow;
    into.setAttribute("style", `background-color:${tint}`);
  }
};

// Only a picture from the web shows in another app; one of Tisty's own files leaves its words.
const pictured = (from: Element, page: Document): Node => {
  const src = from.getAttribute("src") ?? "";
  const alt = from.getAttribute("alt") ?? "";
  if (!/^https?:/i.test(src)) return page.createTextNode(alt);
  const copy = page.createElement("img");
  copy.setAttribute("src", src);
  copy.setAttribute("alt", alt);
  return copy;
};

// Tisty draws a task's box with its stylesheet, which never leaves with the html.
const checkOf = (one: Element): string | null => {
  if (!one.classList.contains("check") || !one.hasAttribute("data-done")) return null;
  return one.getAttribute("data-done") === "true" ? "☑ " : "☐ ";
};

// An icon left out between two words would leave both their spaces side by side, wherever the next words are drawn.
type Gap = { dropped: boolean; open: boolean };

const appended = (into: Node, words: string, gap: Gap, page: Document) => {
  const said = gap.dropped && gap.open ? words.replace(/^[ \t]+/, "") : words;
  if (!said) return;
  into.appendChild(page.createTextNode(said));
  gap.dropped = false;
  gap.open = /\s$/.test(said);
};

const cleaned = (
  from: Node,
  into: Node,
  page: Document,
  gap: Gap = { dropped: false, open: false },
) => {
  for (const one of Array.from(from.childNodes)) {
    if (one.nodeType === TEXT) {
      appended(into, (one.textContent ?? "").replace(UNSEEN, ""), gap, page);
      continue;
    }
    if (one.nodeType !== ELEMENT) continue;
    const element = one as Element;
    const check = checkOf(element);
    if (check) {
      appended(into, check, gap, page);
      continue;
    }
    if (element.matches(FURNITURE)) {
      gap.dropped = true;
      continue;
    }
    // A card for another document or task leaves as its title, as a link into Tisty does.
    if (element.matches(".doc-card")) {
      appended(into, element.querySelector(".card-name")?.textContent ?? "", gap, page);
      continue;
    }
    if (element.localName === "img") {
      into.appendChild(pictured(element, page));
      gap.dropped = false;
      gap.open = false;
      continue;
    }
    // A link into Tisty leads nowhere outside it, so only its words go.
    const outward = element.tagName !== "A" || LINKS.test(element.getAttribute("href") ?? "");
    if (!(KEPT.has(element.tagName) || MATH.has(element.localName)) || !outward) {
      cleaned(element, into, page, gap);
      continue;
    }
    const copy = page.createElement(element.localName);
    kept(element, copy);
    // A new block starts its own line, so a gap left at the end of the one before trims nothing in it.
    cleaned(
      element,
      copy,
      page,
      BLOCKS.has(element.tagName) ? { dropped: false, open: false } : gap,
    );
    into.appendChild(copy);
  }
};

const cellBreak = (one: Element) => (one.nextElementSibling ? "\t" : "");

const grouped = (part: Element): string => {
  const out: string[] = [];
  saidOne(part, out, 0);
  const words = out.join("").trim();
  return /^[\p{L}\p{N}]*$/u.test(words) ? words : `(${words})`;
};

const lineStarted = (out: string[]) => {
  const last = out[out.length - 1];
  if (last !== undefined && !last.endsWith("\n")) out.push("\n");
};

// What a reader without formatting sees: one line per block, a mark before each item, a tab between cells.
const said = (from: Node, out: string[], depth: number) => {
  for (const one of Array.from(from.childNodes)) saidOne(one, out, depth);
};

// Code keeps every line and blank it was written with, so its breaks stay apart from the ones tidied between blocks.
// A parsed page never holds a NUL, where any private-use letter could be a glyph someone wrote.
const CODE_LINE = String.fromCharCode(0);

const saidOne = (one: Node, out: string[], depth: number) => {
  if (one.nodeType === TEXT) {
    out.push((one.textContent ?? "").replace(/\s*\n\s*/g, " "));
    return;
  }
  if (one.nodeType !== ELEMENT) return;
  const element = one as Element;
  const tag = element.tagName;
  const marks = MATH_SAID[element.localName];
  if (tag === "BR") out.push("\n");
  else if (tag === "PRE")
    out.push("\n", (element.textContent ?? "").replace(/\n$/, "").replace(/\n/g, CODE_LINE), "\n");
  else if (tag === "HR") out.push("\n---\n");
  else if (tag === "IMG") out.push(element.getAttribute("alt") ?? "");
  else if (marks && element.children.length === marks.length + 1) {
    const [base, ...rest] = Array.from(element.children);
    out.push(grouped(base) + rest.map((part, at) => marks[at] + grouped(part)).join(""));
  } else if (tag === "LI") item(element, out, depth);
  // A list inside an item and a row of a table follow on the next line, with no blank line between.
  else if ((tag === "UL" || tag === "OL") && depth > 0) said(element, out, depth);
  else if (tag === "TR") {
    lineStarted(out);
    said(element, out, depth);
    lineStarted(out);
  } else if (tag === "TD" || tag === "TH") {
    said(element, out, depth);
    out.push(cellBreak(element));
  } else {
    const block = BLOCKS.has(tag);
    if (block) out.push("\n");
    said(element, out, depth);
    if (block) out.push("\n");
  }
};

const item = (element: Element, out: string[], depth: number) => {
  const list = element.parentElement;
  const at = list ? Array.from(list.children).indexOf(element) : 0;
  const start = Number(list?.getAttribute("start") ?? 1);
  const mark = list?.tagName === "OL" ? `${start + at}. ` : "- ";
  lineStarted(out);
  out.push("  ".repeat(depth), mark);
  said(element, out, depth + 1);
  lineStarted(out);
};

// A quote or a figure says nothing before its first line, so code inside one still opens the copy.
const OPENS_SILENT = new Set(["BLOCKQUOTE", "FIGURE", "DIV"]);

const opensWithCode = (shown: Node): boolean => {
  const first = Array.from(shown.childNodes).find(
    (one) => one.nodeType === ELEMENT || (one.textContent ?? "").trim() !== "",
  ) as Element | undefined;
  if (first?.tagName === "PRE") return true;
  return first !== undefined && OPENS_SILENT.has(first.tagName) && opensWithCode(first);
};

const plainOf = (shown: Node): string => {
  const out: string[] = [];
  said(shown, out, 0);
  // A no-break space only holds a gap on the page; a reader, as a terminal, needs the plain space it stands for.
  const tidied = out
    .join("")
    .replace(/\u00A0/g, " ")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n");
  // Code keeps the indent its first line opens with; anything else starts at its first word.
  const started = opensWithCode(shown) ? tidied.replace(/^\n+/, "") : tidied.trimStart();
  return started.split(CODE_LINE).join("\n").trimEnd();
};

// The element a copy stops climbing at: the page itself, or the document when there is none.
const isPage = (one: Element): boolean =>
  one.matches(".page") || one.tagName === "BODY" || one.tagName === "HTML";

// A numbered list copied from its third item goes on counting from there.
const startFrom = (list: Element, copy: Element, opening: Node) => {
  if (list.tagName !== "OL") return;
  const first = Array.from(list.children).findIndex((one) => one.contains(opening));
  if (first <= 0) return;
  copy.setAttribute("start", String(Number(list.getAttribute("start") ?? 1) + first));
};

/** Whole blocks chosen on a page, with the items of one list kept in that list and its count. */
export const chosenWhole = (
  chosen: readonly Element[],
  page: Document = document,
): DocumentFragment => {
  const out = page.createDocumentFragment();
  let from: Element | null = null;
  let into: Element | null = null;
  for (const one of chosen) {
    const parent = one.parentElement;
    if (one.tagName !== "LI" || !parent || !/^[OU]L$/.test(parent.tagName)) {
      from = null;
      out.append(one.cloneNode(true));
      continue;
    }
    if (from !== parent || !into) {
      into = parent.cloneNode(false) as Element;
      startFrom(parent, into, one);
      out.append(into);
      from = parent;
    }
    into.append(one.cloneNode(true));
  }
  return out;
};

// Words chosen inside one paragraph, heading, item or cell leave as words, as a browser copies them, not as that block.
const TEXT_BLOCKS = new Set([
  "P",
  "H1",
  "H2",
  "H3",
  "H4",
  "H5",
  "H6",
  "LI",
  "TD",
  "TH",
  "DT",
  "DD",
  "SUMMARY",
]);

// The node a range opens on: the child it points at when it is set on an element, as a triple-click leaves it.
const openingOf = (range: Range): Node => {
  const { startContainer, startOffset } = range;
  return startContainer.nodeType === ELEMENT
    ? (startContainer.childNodes[startOffset] ?? startContainer)
    : startContainer;
};

// A list only partly chosen is cloned bare at the range's start; it still counts from the item the range opens in.
const countedOnEdge = (inner: DocumentFragment, range: Range, opening: Node) => {
  const chain: Element[] = [];
  for (
    let one = opening.parentElement;
    one && one !== range.commonAncestorContainer;
    one = one.parentElement
  )
    chain.unshift(one);
  let copy: Node | null = inner.firstChild;
  for (const original of chain) {
    if (!copy || copy.nodeType !== ELEMENT) return;
    startFrom(original, copy as Element, opening);
    copy = copy.firstChild;
  }
};

/** The words a range holds, inside the code block, table, list or quote they were chosen in, which a bare clone leaves behind. */
export const chosenContents = (range: Range): DocumentFragment => {
  const inner = range.cloneContents();
  const opening = openingOf(range);
  countedOnEdge(inner, range, opening);
  const page = range.startContainer.ownerDocument ?? document;
  const common = range.commonAncestorContainer;
  let around: Element | null =
    common.nodeType === ELEMENT ? (common as Element) : common.parentElement;
  let held: Node = inner;
  while (around && !isPage(around) && !TEXT_BLOCKS.has(around.tagName)) {
    // A formula's drawing or the editor's own furniture would drop every word wrapped in it.
    if (!around.matches(FURNITURE)) {
      const copy = around.cloneNode(false) as Element;
      startFrom(around, copy, opening);
      copy.append(held);
      held = copy;
    }
    around = around.parentElement;
  }
  const out = page.createDocumentFragment();
  out.append(held);
  return out;
};

/** What the words chosen on a page are to another app, from the page the core drew for them. */
export const copiedAs = (words: string, shown: DocumentFragment): Copied => {
  const page = shown.ownerDocument;
  const holder = page.createElement("div");
  cleaned(shown, holder, page);
  const plain = plainOf(holder);
  // Blocks left with nothing in them would still paste as empty lines.
  const shows = plain !== "" || holder.querySelector("img, hr, math") !== null;
  return { tisty: words, html: shows ? holder.innerHTML : "", plain };
};

const written = (data: DataTransfer, copied: Copied) => {
  data.setData(TISTY_MARKDOWN, copied.tisty);
  data.setData("text/plain", copied.plain);
  data.setData("text/html", copied.html);
};

/** Fills a copy or cut event with the words in every face, while the event still lets the page write. */
export const clippedWords = (
  event: ClipboardEvent,
  words: string,
  shown: DocumentFragment,
): void => {
  const data = event.clipboardData;
  if (!data) return;
  event.preventDefault();
  written(data, copiedAs(words, shown));
};

// A browser offers copy with nothing chosen only to a page that claims it before.
const claimed = (event: Event) => event.preventDefault();

// Only a copy event lets a page name a type of its own, so the menu raises one of its own.
const rewritten = (page: Document, copied: Copied): boolean => {
  const view = page.defaultView;
  if (!view) return false;
  let filled = false;
  const fills = (event: ClipboardEvent) => {
    if (!event.clipboardData) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    written(event.clipboardData, copied);
    filled = true;
  };
  view.addEventListener("beforecopy", claimed, true);
  view.addEventListener("copy", fills, true);
  try {
    page.execCommand("copy");
  } catch {
    return false;
  } finally {
    view.removeEventListener("beforecopy", claimed, true);
    view.removeEventListener("copy", fills, true);
  }
  return filled;
};

/** The menu's copy: every face through a copy event, or html and words alone when the page cannot raise one. */
export const copiedWords = async (
  words: string,
  shown: DocumentFragment,
  page: Document = document,
): Promise<void> => {
  const copied = copiedAs(words, shown);
  if (rewritten(page, copied)) return;
  await copiedHtml(copied.html, copied.plain);
};
