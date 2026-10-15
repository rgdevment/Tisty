import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { docDrawn, openLink } from "../core";
import { markup } from "../glyphs";
import { serve } from "../inUse";
import { t } from "../locales";
import { pictured } from "../previews";
import { colored, dressed, heads, saidKinds, shell, went } from "./paging";
import { type Lending, lend } from "./widgeting";
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

const fitted = (page: Document, wide?: number, zoom?: number) => {
  const root = page.documentElement;
  if (wide) root.style.setProperty("--wide", `${wide}px`);
  root.style.setProperty("zoom", String(zoom ?? 1));
};

const iconed = (page: Document) => {
  for (const one of page.querySelectorAll<HTMLElement>("span.ico[data-ico]")) {
    const drawn = markup(one.getAttribute("data-ico") ?? "");
    if (!drawn) continue;
    const hue = one.getAttribute("data-hue");
    if (hue) one.classList.add(`ico-${hue}`);
    const glyph = page.createElementNS("http://www.w3.org/2000/svg", "svg");
    glyph.setAttribute("viewBox", "0 0 24 24");
    glyph.setAttribute("aria-hidden", "true");
    glyph.innerHTML = drawn;
    one.replaceChildren(glyph);
  }
};

const pictures = (page: Document) => {
  for (const img of page.querySelectorAll<HTMLImageElement>(".page img[src]")) {
    const at = img.getAttribute("src") ?? "";
    if (!at || /^(https?|data|asset|blob|file):/i.test(at) || !pictured(at)) continue;
    img.removeAttribute("src");
    serve(at)
      .then((real) => img.setAttribute("src", convertFileSrc(real)))
      .catch(() => undefined);
  }
};

const widgets = (page: Document, stranger: boolean): Lending[] => {
  if (stranger) return [];
  const lent: Lending[] = [];
  for (const code of page.querySelectorAll<HTMLElement>(
    "figure.code > pre > code.language-widget",
  )) {
    const source = code.textContent ?? "";
    const figure = code.closest("figure");
    if (!figure || !source.trim()) continue;
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
  const [html, setHtml] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    docDrawn(body)
      .then((drawn) => live && setHtml(shell(drawn)))
      .catch(() => live && setHtml(shell("")));
    return () => {
      live = false;
    };
  }, [body]);

  useEffect(() => {
    const held = frame.current;
    if (!held || html === null) return;
    let lent: Lending[] = [];
    let live = true;
    const retheme = () => {
      if (held.contentDocument) dressed(held.contentDocument);
      for (const one of lent) one.retheme();
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
      if (asked.kind === "block") now.onBlock?.(asked.at);
    };
    const refused = (event: Event) => event.preventDefault();
    const loaded = () => {
      const page = held.contentDocument;
      if (!page) return;
      dressed(page);
      fitted(page, reach.current.wide, reach.current.zoom);
      saidKinds(page, (kind) => t(`said${kind}` as Parameters<typeof t>[0]));
      iconed(page);
      pictures(page);
      lent = widgets(page, stranger);
      colored(page, () => live);
      page.addEventListener("click", clicked);
      page.addEventListener("auxclick", refused);
      page.addEventListener("submit", refused);
      page.addEventListener("widgetdoc", (event) => {
        const id = (event as CustomEvent<string>).detail;
        if (id) reach.current.onDoc(id);
      });
      reach.current.onOutline?.(heads(page));
      reach.current.onShown?.(held.contentWindow);
    };
    const watching = new MutationObserver(retheme);
    watching.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    held.addEventListener("load", loaded);
    held.srcdoc = html;
    return () => {
      live = false;
      watching.disconnect();
      held.removeEventListener("load", loaded);
      for (const one of lent) one.drop();
      reach.current.onShown?.(null);
    };
  }, [html, stranger]);

  useEffect(() => {
    const page = frame.current?.contentDocument;
    if (page?.querySelector(".page")) fitted(page, wide, zoom);
  }, [wide, zoom]);

  return (
    <iframe
      ref={frame}
      title={label}
      className="block min-h-0 w-full flex-1 border-0 bg-sheet"
      referrerPolicy="no-referrer"
    />
  );
}
