import { GHOST } from "./pageSpots";

export const FIELDS = "input, textarea, select";

const LITERAL = "pre, code, kbd, figure.code > figcaption";

export const WRITABLE =
  ".page > :is(p, h1, h2, h3, h4, h5, h6, ul, ol, blockquote, figure.code, .footnote, details)";

export const elementOf = (node: Node | null | undefined) =>
  node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : node?.parentElement;

export const ghostly = (node: Node) => Boolean(elementOf(node)?.closest(GHOST));

export const unwritable = (node: Node) =>
  Boolean(elementOf(node)?.closest('.page > [contenteditable="false"]'));

export const inCode = (node: Node) => Boolean(elementOf(node)?.closest("code"));

// A field laid over the page or inside it, as a link's address or a table's cell, writes its own words.
export const onSheet = (page: Document, event: Event) => {
  const target = event.target as Element | null;
  if (target?.closest?.(FIELDS)) return false;
  return Boolean(page.querySelector(".page")?.contains(target));
};

// A caret on an empty line measures nothing, so the line's own element stands for it, never the whole sheet.
// A caret between letters often measures nothing, so a letter beside it tells which line it stands on.
export const caretBox = (range: Range): DOMRect | null => {
  const box =
    typeof range.getBoundingClientRect === "function" ? range.getBoundingClientRect() : null;
  if (box && (box.width || box.height)) return box;
  const node = range.startContainer;
  if (node.nodeType !== Node.TEXT_NODE || !node.ownerDocument) return null;
  const length = (node as Text).length;
  const at = range.startOffset;
  if (!length) return null;
  const probe = node.ownerDocument.createRange();
  probe.setStart(node, at < length ? at : at - 1);
  probe.setEnd(node, at < length ? at + 1 : at);
  const letter = probe.getClientRects?.()[0] ?? null;
  return letter?.height ? letter : null;
};

export const lineOfCaret = (range: Range | null | undefined): Element | null => {
  const node = range?.startContainer;
  if (!node) return null;
  const element = elementOf(node);
  const inside = element?.matches(".page") ? element.childNodes[range.startOffset] : null;
  const line = inside?.nodeType === Node.ELEMENT_NODE ? (inside as Element) : element;
  return line && !line.matches(".page") ? line : null;
};

// The sheet is written on whole, but a table, a drawing or a rule is changed by its own hands.
// A task's labels are filled under, not renamed, until its sections are asked for; then only they are written.
export const opened = (page: Document) => {
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
