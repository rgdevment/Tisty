import { convertFileSrc } from "@tauri-apps/api/core";
import { openLink } from "../core";
import { markup } from "../glyphs";
import { serve, showing } from "../inUse";
import { t } from "../locales";
import { named, pictured } from "../previews";
import { charsAt, colored, HUES, saidKinds, type Went, went } from "./paging";
import { type Lending, lend, locked } from "./widgeting";

export type Reaching = {
  readonly body: string;
  readonly onDoc: (file: string) => void;
  readonly onOpen?: (reference: string) => void;
  readonly onTag?: (name: string) => void;
  readonly onBlock?: (at: [number, number]) => void;
};

export type Worn = { lent: Lending[]; kept: (() => void)[] };

export const fitted = (page: Document, wide?: number, zoom?: number) => {
  const root = page.documentElement;
  if (wide) root.style.setProperty("--wide", `${wide}px`);
  root.style.setProperty("zoom", String(zoom ?? 1));
};

export const iconed = (page: Document) => {
  for (const one of page.querySelectorAll<HTMLElement>("span.ico[data-ico]")) {
    const drawn = markup(one.dataset.ico ?? "");
    if (!drawn) continue;
    const hue = one.dataset.hue ?? "";
    if (HUES.includes(hue)) one.classList.add(`ico-${hue}`);
    const glyph = page.createElementNS("http://www.w3.org/2000/svg", "svg");
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
    if (!at || /^(https?|data|asset|blob|file):/i.test(at)) continue;
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

export const widgets = (page: Document, stranger: boolean): Lending[] => {
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
    figure.replaceWith(drawn);
    lent.push(lend(drawn, source));
  }
  return lent;
};

export const worn = (page: Document, stranger: boolean, live: () => boolean): Worn => {
  saidKinds(page, (kind) => t(`said${kind}` as Parameters<typeof t>[0]));
  iconed(page);
  const kept = pictures(page, live);
  const lent = widgets(page, stranger);
  colored(page, live);
  return { lent, kept };
};

export const answered = (page: Document, asked: Went, reach: Reaching) => {
  if (!asked) return;
  if (asked.kind === "doc") reach.onDoc(asked.id);
  if (asked.kind === "web") openLink(asked.href).catch(() => {});
  if (asked.kind === "here") page.getElementById(asked.id)?.scrollIntoView({ block: "start" });
  if (asked.kind === "attachment") reach.onOpen?.(asked.reference);
  if (asked.kind === "tag") reach.onTag?.(asked.name);
  if (asked.kind === "block")
    reach.onBlock?.([charsAt(reach.body, asked.at[0]), charsAt(reach.body, asked.at[1])]);
};

export const wired = (page: Document, reach: () => Reaching) => {
  const refused = (event: Event) => event.preventDefault();
  page.addEventListener("click", (event) => {
    const target = event.target as Element | null;
    if (event.button !== 0 || typeof target?.closest !== "function") return;
    const selecting = !(page.getSelection()?.isCollapsed ?? true);
    if (target.closest("a")) event.preventDefault();
    answered(page, went(target, selecting), reach());
  });
  for (const kind of ["auxclick", "submit", "dragstart", "contextmenu"]) {
    page.addEventListener(kind, refused);
  }
  // The window's own shortcuts listen on the app, which never hears a key pressed in the page.
  page.addEventListener("keydown", (event) => {
    const passed = new KeyboardEvent("keydown", event);
    window.dispatchEvent(passed);
    if (passed.defaultPrevented) event.preventDefault();
  });
  page.addEventListener("widgetdoc", (event) => {
    const id = (event as CustomEvent<string>).detail;
    if (id) reach().onDoc(id);
  });
};
