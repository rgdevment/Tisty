import { convertFileSrc } from "@tauri-apps/api/core";
import { openLink } from "../core";
import { markup } from "../glyphs";
import { serve, showing } from "../inUse";
import { t } from "../locales";
import { named, pictured } from "../previews";
import { type Carding, carded } from "./carding";
import { charsAt, colored, HUES, saidKinds, type Went, went } from "./paging";
import { webCarded } from "./webcarding";
import { type Lending, lend, locked } from "./widgeting";

export type Reaching = Carding & {
  readonly body: string;
  readonly onDoc: (file: string) => void;
  readonly onOpen?: (reference: string) => void;
  readonly onTag?: (name: string) => void;
  readonly onBlock?: (
    at: [number, number],
    block: HTMLElement,
    seed?: string,
    read?: string,
  ) => void;
  readonly onApart?: (source: string) => void;
  readonly onHere?: (id: string) => void;
  readonly onAway?: () => void;
  readonly onTick?: (at: [number, number]) => void;
};

const sources = new WeakMap<Element, string>();

const asking = (page: Document, asked: string, said: string): HTMLButtonElement => {
  const one = page.createElement("button");
  one.type = "button";
  one.dataset.asked = asked;
  one.textContent = said;
  return one;
};

export type Worn = { lent: Lending[]; kept: (() => void)[] };

export const fitted = (page: Document, wide?: number, zoom?: number) => {
  const root = page.documentElement;
  if (wide) root.style.setProperty("--wide", `${wide}px`);
  root.style.setProperty("zoom", String(zoom ?? 1));
};

export const iconed = (page: Document | HTMLElement) => {
  const made = "createElementNS" in page ? page : page.ownerDocument;
  for (const one of page.querySelectorAll<HTMLElement>("span.ico[data-ico]")) {
    const drawn = markup(one.dataset.ico ?? "");
    if (!drawn) continue;
    const hue = one.dataset.hue ?? "";
    if (HUES.includes(hue)) one.classList.add(`ico-${hue}`);
    const glyph = made.createElementNS("http://www.w3.org/2000/svg", "svg");
    glyph.setAttribute("class", "glyph");
    glyph.setAttribute("viewBox", "0 0 24 24");
    glyph.setAttribute("aria-hidden", "true");
    glyph.innerHTML = drawn;
    one.replaceChildren(glyph);
  }
};

export const pictures = (page: Document, live: () => boolean): (() => void)[] => {
  const held: (() => void)[] = [];
  for (const img of page.querySelectorAll<HTMLImageElement>(".page img[src]")) {
    const at = img.getAttribute("src") ?? "";
    if (!at || /^(https?|data|asset|blob|file|tauri):/i.test(at)) continue;
    if (!pictured(at)) {
      const card = page.createElement("a");
      card.className = "card";
      card.setAttribute("href", at);
      card.textContent = img.getAttribute("alt") || named(at);
      img.replaceWith(card);
      continue;
    }
    img.removeAttribute("src");
    held.push(showing(at));
    serve(at)
      .then((real) => live() && img.setAttribute("src", convertFileSrc(real)))
      .catch(() => undefined);
  }
  return held;
};

export const widgets = (page: Document, stranger: boolean, reach?: Reaching): Lending[] => {
  const lent: Lending[] = [];
  for (const code of page.querySelectorAll<HTMLElement>(
    "figure.code > pre > code.language-widget",
  )) {
    const source = code.textContent ?? "";
    const figure = code.closest("figure");
    if (!figure || !source.trim()) continue;
    if (stranger) {
      figure.before(locked());
      continue;
    }
    const drawn = page.createElement("figure");
    drawn.className = "widget";
    drawn.dataset.at = figure.dataset.at ?? "";
    sources.set(drawn, source);
    const bar = page.createElement("div");
    bar.className = "widget-bar";
    const name = page.createElement("span");
    name.className = "widget-name";
    name.textContent = figure.querySelector("figcaption")?.textContent ?? "";
    bar.append(name);
    if (reach?.onBlock) bar.append(asking(page, "code", t("codeShow")));
    if (reach?.onApart) bar.append(asking(page, "apart", t("viewApart")));
    const held = page.createElement("div");
    drawn.append(bar, held);
    figure.replaceWith(drawn);
    lent.push(lend(held, source));
  }
  return lent;
};

const titled = (page: Document) => {
  const sheet = page.querySelector<HTMLElement>(".page");
  if (!sheet) return;
  sheet.setAttribute("data-empty", t("pageEmpty"));
  if (sheet.firstElementChild) return;
  const slot = page.createElement("h1");
  slot.dataset.title = t("titleEmpty");
  sheet.prepend(slot);
};

export const worn = (
  page: Document,
  stranger: boolean,
  live: () => boolean,
  reach?: Reaching,
): Worn => {
  saidKinds(page, (kind) => t(`said${kind}` as Parameters<typeof t>[0]));
  if (reach?.onBlock) titled(page);
  page.documentElement.classList.toggle("ticking", !!reach?.onTick);
  iconed(page);
  const kept = pictures(page, live);
  const lent = widgets(page, stranger, reach);
  if (reach && !stranger) {
    carded(page, reach, live);
    webCarded(page, live);
  }
  colored(page, live);
  return { lent, kept };
};

export const answered = (page: Document, asked: Went, reach: Reaching, read = "") => {
  if (!asked) return;
  if (asked.kind === "doc") reach.onDoc(asked.id);
  if (asked.kind === "web") openLink(asked.href).catch(() => {});
  if (asked.kind === "here") {
    const at = page.getElementById(asked.id);
    if (at) at.scrollIntoView({ block: "start" });
    else reach.onHere?.(asked.id);
  }
  if (asked.kind === "attachment") reach.onOpen?.(asked.reference);
  if (asked.kind === "tag") reach.onTag?.(asked.name);
  if (asked.kind === "tick")
    reach.onTick?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])]);
  if (asked.kind === "block") {
    const block = page.querySelector<HTMLElement>(
      `.page > [data-at="${asked.at[0]}-${asked.at[1]}"]`,
    );
    const at: [number, number] = [
      charsAt(reach.body, asked.at[0]),
      charsAt(reach.body, asked.at[1]),
    ];
    if (block) reach.onBlock?.(at, block, "", read);
  }
  if (asked.kind === "title" && reach.onBlock) {
    const slot = page.querySelector<HTMLElement>(".page > h1[data-title]");
    if (slot) reach.onBlock([0, 0], slot, "# ");
  }
  if (asked.kind === "end" && reach.onBlock) {
    const sheet = page.querySelector(".page");
    if (!sheet) return;
    const fresh = page.createElement("p");
    sheet.append(fresh);
    reach.onBlock([reach.body.length, reach.body.length], fresh);
    if (fresh.isConnected) fresh.remove();
  }
  if (asked.kind === "apart") {
    const drawn = page.querySelector(
      `.page > figure.widget[data-at="${asked.at[0]}-${asked.at[1]}"]`,
    );
    const source = drawn ? sources.get(drawn) : undefined;
    if (source !== undefined) reach.onApart?.(source);
  }
};

const below = (page: Document, target: Element, event: MouseEvent): boolean => {
  const sheet = page.querySelector(".page");
  if (!sheet || (target !== sheet && target !== page.body && target !== page.documentElement))
    return false;
  const last = sheet.lastElementChild;
  return !last || event.clientY > last.getBoundingClientRect().bottom;
};

const readTo = (page: Document, event: MouseEvent): string => {
  const block = (event.target as Element | null)?.closest?.(".page > [data-at]");
  const caret = page.caretRangeFromPoint?.(event.clientX, event.clientY);
  if (!block || !caret || !block.contains(caret.startContainer)) return "";
  const range = page.createRange();
  range.setStart(block, 0);
  range.setEnd(caret.startContainer, caret.startOffset);
  return range.toString();
};

export const wired = (page: Document, reach: () => Reaching) => {
  const refused = (event: Event) => event.preventDefault();
  page.addEventListener("click", (event) => {
    const target = event.target as Element | null;
    if (event.button !== 0 || typeof target?.closest !== "function") return;
    const selecting = !(page.getSelection()?.isCollapsed ?? true);
    if (target.closest("a")) event.preventDefault();
    const asked = went(target, selecting) ?? (below(page, target, event) ? { kind: "end" } : null);
    if (!asked && !target.closest(".editing")) reach().onAway?.();
    answered(page, asked, reach(), asked?.kind === "block" ? readTo(page, event) : "");
  });
  for (const kind of ["auxclick", "submit", "dragstart", "contextmenu"]) {
    page.addEventListener(kind, refused);
  }
  // The window's own shortcuts listen on the app, which never hears a key pressed in the page.
  page.addEventListener("keydown", (event) => {
    const writing = (event.target as Element | null)?.closest?.(".editing");
    const asked = event.ctrlKey || event.metaKey || event.key === "F1";
    if (writing && (event.defaultPrevented || !asked)) return;
    const passed = new KeyboardEvent("keydown", event);
    window.dispatchEvent(passed);
    if (passed.defaultPrevented) event.preventDefault();
  });
  page.addEventListener("widgetdoc", (event) => {
    const id = (event as CustomEvent<string>).detail;
    if (id) reach().onDoc(id);
  });
};
