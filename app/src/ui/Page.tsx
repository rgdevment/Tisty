import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { docDrawn, openLink } from "../core";
import { markup } from "../glyphs";
import { serve, showing } from "../inUse";
import { t } from "../locales";
import { named, pictured } from "../previews";
import { charsAt, colored, dressed, escaped, HUES, heads, saidKinds, shell, went } from "./paging";
import { type Lending, lend, locked } from "./widgeting";
import type { Head } from "./writing";

type Props = {
  body: string;
  label: string;
  stranger: boolean;
  wide?: number;
  zoom?: number;
  onDoc: (file: string) => void;
  onOpen?: (reference: string) => void;
  onTag?: (name: string) => void;
  onBlock?: (at: [number, number]) => void;
  onOutline?: (heads: Head[]) => void;
  onShown?: (page: Window | null) => void;
};

type Drawn = { html: string; turn: number };

const fitted = (page: Document, wide?: number, zoom?: number) => {
  const root = page.documentElement;
  if (wide) root.style.setProperty("--wide", `${wide}px`);
  root.style.setProperty("zoom", String(zoom ?? 1));
};

const iconed = (page: Document) => {
  for (const one of page.querySelectorAll<HTMLElement>("span.ico[data-ico]")) {
    const drawn = markup(one.getAttribute("data-ico") ?? "");
    if (!drawn) continue;
    const hue = one.getAttribute("data-hue") ?? "";
    if (HUES.includes(hue)) one.classList.add(`ico-${hue}`);
    const glyph = page.createElementNS("http://www.w3.org/2000/svg", "svg");
    glyph.setAttribute("viewBox", "0 0 24 24");
    glyph.setAttribute("aria-hidden", "true");
    glyph.innerHTML = drawn;
    one.replaceChildren(glyph);
  }
};

const pictures = (page: Document, live: () => boolean): (() => void)[] => {
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

const widgets = (page: Document, stranger: boolean): Lending[] => {
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
    drawn.setAttribute("data-at", figure.getAttribute("data-at") ?? "");
    figure.replaceWith(drawn);
    lent.push(lend(drawn, source));
  }
  return lent;
};

export default function Page(props: Props) {
  const { body, label, stranger, wide, zoom } = props;
  const frame = useRef<HTMLIFrameElement>(null);
  const reach = useRef(props);
  reach.current = props;
  const [drawn, setDrawn] = useState<Drawn | null>(null);
  const scrolled = useRef(0);

  useEffect(() => {
    let live = true;
    const shown = (html: string) =>
      live && setDrawn((was) => ({ html: shell(html), turn: (was?.turn ?? 0) + 1 }));
    docDrawn(body)
      .then(shown)
      .catch(() => shown(`<pre>${escaped(body)}</pre>`));
    return () => {
      live = false;
    };
  }, [body]);

  useEffect(() => {
    const held = frame.current;
    if (!held || !drawn) return;
    let lent: Lending[] = [];
    let kept: (() => void)[] = [];
    let live = true;
    const alive = () => live;
    const retheme = () => {
      if (held.contentDocument) dressed(held.contentDocument);
    };
    const clicked = (event: MouseEvent) => {
      const page = held.contentDocument;
      const target = event.target as Element | null;
      if (!page || event.button !== 0 || typeof target?.closest !== "function") return;
      const selecting = !(page.getSelection()?.isCollapsed ?? true);
      const asked = went(target, selecting);
      if (target.closest("a")) event.preventDefault();
      if (!asked) return;
      const now = reach.current;
      if (asked.kind === "doc") now.onDoc(asked.id);
      if (asked.kind === "web") openLink(asked.href).catch(() => {});
      if (asked.kind === "here") page.getElementById(asked.id)?.scrollIntoView({ block: "start" });
      if (asked.kind === "attachment") now.onOpen?.(asked.reference);
      if (asked.kind === "tag") now.onTag?.(asked.name);
      if (asked.kind === "block")
        now.onBlock?.([charsAt(now.body, asked.at[0]), charsAt(now.body, asked.at[1])]);
    };
    const refused = (event: Event) => event.preventDefault();
    // The window's own shortcuts listen on the app, which never hears a key pressed in the page.
    const pressed = (event: KeyboardEvent) => {
      const passed = new KeyboardEvent("keydown", event);
      window.dispatchEvent(passed);
      if (passed.defaultPrevented) event.preventDefault();
    };
    const widgetDoc = (event: Event) => {
      const id = (event as CustomEvent<string>).detail;
      if (id) reach.current.onDoc(id);
    };
    const loaded = () => {
      const page = held.contentDocument;
      if (!page?.querySelector(".page")) return;
      page.addEventListener("click", clicked);
      page.addEventListener("auxclick", refused);
      page.addEventListener("submit", refused);
      page.addEventListener("dragstart", refused);
      page.addEventListener("contextmenu", refused);
      page.addEventListener("keydown", pressed);
      page.addEventListener("widgetdoc", widgetDoc);
      page.querySelector(".page")?.removeAttribute("inert");
      dressed(page);
      fitted(page, reach.current.wide, reach.current.zoom);
      reach.current.onOutline?.(heads(page));
      reach.current.onShown?.(held.contentWindow);
      saidKinds(page, (kind) => t(`said${kind}` as Parameters<typeof t>[0]));
      iconed(page);
      kept = pictures(page, alive);
      lent = widgets(page, stranger);
      colored(page, alive);
      held.contentWindow?.scrollTo(0, scrolled.current);
    };
    const watching = new MutationObserver(retheme);
    watching.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    held.addEventListener("load", loaded);
    held.srcdoc = drawn.html;
    return () => {
      live = false;
      scrolled.current = held.contentWindow?.scrollY ?? 0;
      watching.disconnect();
      held.removeEventListener("load", loaded);
      for (const one of lent) one.drop();
      for (const one of kept) one();
      reach.current.onShown?.(null);
    };
  }, [drawn, stranger]);

  useEffect(() => {
    const page = frame.current?.contentDocument;
    if (page?.querySelector(".page")) fitted(page, wide, zoom);
  }, [wide, zoom]);

  return (
    <iframe
      key={drawn?.turn ?? 0}
      ref={frame}
      title={label}
      className="block min-h-0 w-full flex-1 border-0 bg-sheet"
      referrerPolicy="no-referrer"
    />
  );
}
