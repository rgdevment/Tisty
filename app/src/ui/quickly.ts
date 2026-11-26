import type { formatButtons } from "tisty-editor/edit";
import { t } from "../locales";
import { EMBED } from "./embeds";
import { onMac } from "./macKeys";
import { placeOf } from "./pagePlaces";
import { type WordAt, wordAt, wordIn } from "./pageWordAt";
import { shotAt } from "./paging";

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
  readonly embed: Element | null;
  readonly word: WordAt | null;
};

export type Bar = { readonly buttons: typeof formatButtons; readonly css: string };

export type Quick = {
  readonly bar?: Bar;
  readonly onFormat?: (asked: Formatting) => void;
  readonly onMenu?: (asked: Menued) => void;
};

const blockOf = (node: Node | null): HTMLElement | null => {
  const element = node?.nodeType === 1 ? (node as Element) : (node?.parentElement ?? null);
  return element?.closest<HTMLElement>(".page > [data-at]") ?? null;
};

const askedOnMac = (event: MouseEvent) =>
  onMac && (event.button === 2 || (event.button === 0 && event.ctrlKey));

const codeOf = (node: Node | null | undefined) => {
  const element = node?.nodeType === 1 ? (node as Element) : (node?.parentElement ?? null);
  return element?.closest("figure.code") ?? element?.closest("pre, code") ?? null;
};

// Code is written as it stands and a field writes its own words, so neither takes a format.
export const unformattable = (page: Document): boolean => {
  if (page.activeElement?.closest("input, textarea, select")) return true;
  const chosen = page.getSelection();
  const range = chosen?.rangeCount ? chosen.getRangeAt(0) : null;
  const start = codeOf(range?.startContainer);
  const end = codeOf(range?.endContainer);
  if (start === end) return start !== null;
  return [start, end].some((code) => code !== null && !code.matches("figure.code"));
};

export const picked = (page: Document): Omit<Formatting, "open" | "close"> | null => {
  const chosen = page.getSelection();
  if (!chosen || chosen.isCollapsed || chosen.rangeCount === 0) return null;
  const range = chosen.getRangeAt(0);
  const block = blockOf(range.startContainer);
  if (!block || !blockOf(range.endContainer)) return null;
  const at = placeOf(block);
  const said = chosen.toString();
  if (!at || !said.trim()) return null;
  const before = page.createRange();
  before.setStart(block, 0);
  before.setEnd(range.startContainer, range.startOffset);
  return { at, block, read: before.toString(), said };
};

const barOf = (
  page: Document,
  buttons: Bar["buttons"],
  pick: (open: string, close: string) => void,
) => {
  const bar = page.createElement("div");
  bar.className = "cm-bar quick-bar";
  bar.setAttribute("role", "toolbar");
  bar.setAttribute("aria-label", t("formatting"));
  bar.hidden = true;
  const { formats, pens } = buttons(page, t, pick);
  bar.append(...formats, ...pens);
  page.body.append(bar);
  return bar;
};

export const quickly = (page: Document, reach: () => Quick) => {
  let chosen: Omit<Formatting, "open" | "close"> | null = null;
  const brought = reach().bar;
  const bar =
    brought &&
    barOf(page, brought.buttons, (open, close) => {
      const asked = chosen;
      hide();
      page.getSelection()?.removeAllRanges();
      if (asked) reach().onFormat?.({ ...asked, open, close });
    });
  const hide = () => {
    if (bar) bar.hidden = true;
    chosen = null;
  };
  const show = () => {
    const writing = page.querySelector(".editing") !== null;
    chosen = reach().onFormat && !writing && !unformattable(page) ? picked(page) : null;
    if (!chosen || !bar) return hide();
    const box = page.getSelection()?.getRangeAt(0).getBoundingClientRect();
    const view = page.defaultView;
    if (!box || !view) return hide();
    bar.hidden = false;
    // The page is zoomed as a whole, so what the selection measures is scaled once already.
    const zoom = Number.parseFloat(page.documentElement.style.zoom || "1") || 1;
    const above = (box.top + view.scrollY) / zoom - bar.offsetHeight - 8;
    const below = (box.bottom + view.scrollY) / zoom + 8;
    // Words at the top of the view leave no room above them, so the bar goes under them instead of over them.
    bar.style.top = `${above < view.scrollY / zoom + 4 ? below : above}px`;
    const widest = (view.innerWidth + view.scrollX) / zoom - bar.offsetWidth - 4;
    bar.style.left = `${Math.max(4, Math.min((box.left + view.scrollX) / zoom, widest))}px`;
  };
  page.addEventListener("mouseup", (event) => {
    if (event.button !== 0 || bar?.contains(event.target as Node)) return;
    setTimeout(show);
  });
  page.addEventListener("keyup", () => {
    if (!page.getSelection()?.isCollapsed) show();
  });
  page.addEventListener("selectionchange", () => {
    if (page.getSelection()?.isCollapsed) hide();
    else if (bar && !bar.hidden) show();
  });
  let caretStood: Range | null | undefined;
  let keyed = false;
  page.addEventListener(
    "mousedown",
    (event) => {
      caretStood = undefined;
      keyed = false;
      if (!askedOnMac(event)) return;
      const selection = page.getSelection();
      const range = selection?.rangeCount ? selection.getRangeAt(0) : null;
      if (!range || range.collapsed) caretStood = range?.cloneRange() ?? null;
    },
    true,
  );
  const caretBack = () => {
    const stood = caretStood;
    caretStood = undefined;
    if (stood === undefined) return;
    const selection = page.getSelection();
    selection?.removeAllRanges();
    if (stood) selection?.addRange(stood);
  };
  page.addEventListener(
    "keydown",
    (event) => {
      keyed = event.key === "ContextMenu" || (event.shiftKey && event.key === "F10");
    },
    true,
  );
  // A menu asked from the keyboard comes with no place of its own, so it opens on the caret or the choice.
  const keyedOn = (): { x: number; y: number; range: Range } | null => {
    const asked = keyed;
    keyed = false;
    const selection = page.getSelection();
    const range = asked && selection?.rangeCount ? selection.getRangeAt(0) : null;
    const box = range?.getBoundingClientRect();
    return range && box && (box.width > 0 || box.height > 0)
      ? { x: box.left, y: box.bottom, range }
      : null;
  };
  page.addEventListener("contextmenu", (event) => {
    const pointed = event.target as Element | null;
    // A field the page holds, as the diary's box, keeps the system's menu with its own speller.
    if (pointed?.closest?.("input, textarea, select")) return;
    event.preventDefault();
    caretBack();
    const keyedAt = keyedOn();
    const caretNode = keyedAt?.range.startContainer;
    const target = caretNode
      ? caretNode.nodeType === 1
        ? (caretNode as Element)
        : caretNode.parentElement
      : pointed;
    const onMenu = reach().onMenu;
    if (!onMenu) return;
    const writing = Boolean(target?.closest?.(".editing"));
    const element = writing ? null : blockOf(target);
    const place = placeOf(element);
    const selection = page.getSelection();
    const range = selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
    const under = Boolean(target && range?.intersectsNode(target));
    hide();
    onMenu({
      x: keyedAt?.x ?? event.clientX,
      y: keyedAt?.y ?? event.clientY,
      writing,
      picked: writing || !under ? null : picked(page),
      block: element && place ? { at: place, element } : null,
      picture: writing ? null : shotAt(page, target),
      embed: writing ? null : (target?.closest?.(EMBED) ?? null),
      word: writing
        ? null
        : keyedAt
          ? wordIn(keyedAt.range.startContainer, keyedAt.range.startOffset)
          : wordAt(page, event.clientX, event.clientY),
    });
  });
};
