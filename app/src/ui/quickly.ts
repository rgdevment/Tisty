import { t } from "../locales";
import { formatButtons } from "./barring";
import { shotAt, spanned } from "./paging";

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
  readonly picture: number | null;
};

// What a key pressed over chosen words does to them once the block opens: null keeps them chosen.
export type Chosen = Omit<Formatting, "open" | "close"> & { readonly typed: string | null };

export type Quick = {
  readonly onFormat?: (asked: Formatting) => void;
  readonly onMenu?: (asked: Menued) => void;
  readonly onChoose?: (asked: Chosen) => void;
};

const typedOver = (event: KeyboardEvent): string | null | undefined => {
  if (event.ctrlKey || event.metaKey || event.altKey || event.isComposing) return undefined;
  if (event.key === "Enter") return null;
  if (event.key === "Backspace" || event.key === "Delete") return "";
  return [...event.key].length === 1 ? event.key : undefined;
};

const blockOf = (node: Node | null): HTMLElement | null => {
  const element = node?.nodeType === 1 ? (node as Element) : (node?.parentElement ?? null);
  return element?.closest<HTMLElement>(".page > [data-at]") ?? null;
};

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

const barOf = (page: Document, pick: (open: string, close: string) => void) => {
  const bar = page.createElement("div");
  bar.className = "cm-bar quick-bar";
  bar.setAttribute("role", "toolbar");
  bar.setAttribute("aria-label", t("formatting"));
  bar.hidden = true;
  const { formats, pens } = formatButtons(page, pick);
  bar.append(...formats, ...pens);
  page.body.append(bar);
  return bar;
};

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
    const writing = page.querySelector(".editing") !== null;
    chosen = reach().onFormat && !writing ? picked(page) : null;
    if (!chosen) return hide();
    const box = page.getSelection()?.getRangeAt(0).getBoundingClientRect();
    const view = page.defaultView;
    if (!box || !view) return hide();
    bar.hidden = false;
    // The page is zoomed as a whole, so what the selection measures is scaled once already.
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const top = (box.top + view.scrollY) / zoom - bar.offsetHeight - 8;
    bar.style.top = `${Math.max(view.scrollY / zoom + 4, top)}px`;
    bar.style.left = `${Math.max(4, (box.left + view.scrollX) / zoom)}px`;
  };
  page.addEventListener("mouseup", (event) => {
    if (bar.contains(event.target as Node)) return;
    setTimeout(show);
  });
  // Words chosen on the page are written over as they would be in any editor, the block opening around them.
  page.addEventListener("keydown", (event) => {
    const choose = reach().onChoose;
    const typed = typedOver(event);
    if (!choose || typed === undefined || page.querySelector(".editing")) return;
    const asked = picked(page);
    if (!asked) return;
    event.preventDefault();
    hide();
    choose({ ...asked, typed });
  });
  page.addEventListener("keyup", () => {
    if (!page.getSelection()?.isCollapsed) show();
  });
  page.addEventListener("selectionchange", () => {
    if (page.getSelection()?.isCollapsed) hide();
    else if (!bar.hidden) show();
  });
  page.addEventListener("contextmenu", (event) => {
    event.preventDefault();
    const onMenu = reach().onMenu;
    if (!onMenu) return;
    const target = event.target as Element | null;
    const writing = Boolean(target?.closest?.(".editing"));
    const element = writing ? null : blockOf(target);
    const at = spanned(element?.dataset.at ?? null);
    const selection = page.getSelection();
    const range = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
    const under = Boolean(target && range?.intersectsNode(target));
    hide();
    onMenu({
      x: event.clientX,
      y: event.clientY,
      writing,
      picked: writing || !under ? null : picked(page),
      block: element && at ? { at, element } : null,
      picture: writing ? null : shotAt(page, target),
    });
  });
};
