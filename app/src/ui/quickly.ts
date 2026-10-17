import { markup } from "../glyphs";
import { t } from "../locales";
import { FORMATS, PENS } from "./barring";
import { spanned } from "./paging";

export type Formatting = {
  readonly at: [number, number];
  readonly block: HTMLElement;
  readonly read: string;
  readonly said: string;
  readonly open: string;
  readonly close: string;
};

export type Menued = {
  readonly x: number;
  readonly y: number;
  readonly writing: boolean;
  readonly picked: Omit<Formatting, "open" | "close"> | null;
  readonly block: { readonly at: [number, number]; readonly element: HTMLElement } | null;
};

export type Quick = {
  readonly onFormat?: (asked: Formatting) => void;
  readonly onMenu?: (asked: Menued) => void;
};

const blockOf = (node: Node | null): HTMLElement | null => {
  const element = node?.nodeType === 1 ? (node as Element) : (node?.parentElement ?? null);
  return element?.closest<HTMLElement>(".page > [data-at]") ?? null;
};

// What is selected inside one block of the page, with the words before it, as a click reads them.
export const picked = (page: Document): Omit<Formatting, "open" | "close"> | null => {
  const chosen = page.getSelection();
  if (!chosen || chosen.isCollapsed || chosen.rangeCount === 0) return null;
  const range = chosen.getRangeAt(0);
  const block = blockOf(range.startContainer);
  if (!block || block !== blockOf(range.endContainer)) return null;
  const at = spanned(block.dataset.at ?? null);
  const said = chosen.toString();
  if (!at || !said.trim()) return null;
  const before = page.createRange();
  before.setStart(block, 0);
  before.setEnd(range.startContainer, range.startOffset);
  return { at, block, read: before.toString(), said };
};

const glyph = (page: Document, name: string) => {
  const svg = page.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("aria-hidden", "true");
  svg.innerHTML = markup(name) ?? "";
  return svg;
};

const barOf = (page: Document, pick: (open: string, close: string) => void) => {
  const bar = page.createElement("div");
  bar.className = "cm-bar quick-bar";
  bar.setAttribute("role", "toolbar");
  bar.setAttribute("aria-label", t("formatting"));
  bar.hidden = true;
  const button = (label: string, open: string, close: string) => {
    const one = page.createElement("button");
    one.type = "button";
    one.className = "cm-bar-button";
    one.title = label;
    one.setAttribute("aria-label", label);
    one.addEventListener("mousedown", (event) => event.preventDefault());
    one.addEventListener("click", () => pick(open, close));
    bar.append(one);
    return one;
  };
  for (const [said, , open, close, icon] of FORMATS) {
    const holder = page.createElement("span");
    holder.className = "cm-bar-glyph";
    holder.append(glyph(page, icon));
    button(t(said), open, close).append(holder);
  }
  for (const [hue, said, close] of PENS) {
    const pen = page.createElement("span");
    pen.className = `cm-bar-pen cm-pen-${hue}`;
    button(t(said), "==", close).append(pen);
  }
  page.body.append(bar);
  return bar;
};

// A selection on the page offers the quick formats; a block being written has its own bar.
export const quickly = (page: Document, reach: () => Quick) => {
  let chosen: Omit<Formatting, "open" | "close"> | null = null;
  const bar = barOf(page, (open, close) => {
    const asked = chosen;
    hide();
    page.getSelection()?.removeAllRanges();
    if (asked) reach().onFormat?.({ ...asked, open, close });
  });
  const hide = () => {
    bar.hidden = true;
    chosen = null;
  };
  const show = () => {
    chosen = reach().onFormat ? picked(page) : null;
    if (!chosen) return hide();
    const box = page.getSelection()?.getRangeAt(0).getBoundingClientRect();
    const view = page.defaultView;
    if (!box || !view) return hide();
    bar.hidden = false;
    const top = box.top + view.scrollY - bar.offsetHeight - 8;
    bar.style.top = `${Math.max(view.scrollY + 4, top)}px`;
    bar.style.left = `${Math.max(4, box.left + view.scrollX)}px`;
  };
  page.addEventListener("mouseup", (event) => {
    if (bar.contains(event.target as Node)) return;
    setTimeout(show);
  });
  page.addEventListener("keyup", (event) => {
    if (event.shiftKey) show();
  });
  page.addEventListener("selectionchange", () => {
    if (page.getSelection()?.isCollapsed) hide();
  });
  page.addEventListener("contextmenu", (event) => {
    event.preventDefault();
    const onMenu = reach().onMenu;
    if (!onMenu) return;
    const target = event.target as Element | null;
    const writing = Boolean(target?.closest?.(".editing"));
    const element = writing ? null : blockOf(target);
    const at = spanned(element?.dataset.at ?? null);
    hide();
    onMenu({
      x: event.clientX,
      y: event.clientY,
      writing,
      picked: writing ? null : picked(page),
      block: element && at ? { at, element } : null,
    });
  });
};
