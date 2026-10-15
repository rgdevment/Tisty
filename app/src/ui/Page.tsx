import { useEffect, useRef, useState } from "react";
import { docDrawn } from "../core";
import { fitted, type Reaching, type Worn, wired, worn } from "./dressing";
import { dressed, escaped, heads, shell } from "./paging";
import type { Head } from "./writing";

type Props = Reaching & {
  readonly label: string;
  readonly stranger: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly onOutline?: (heads: Head[]) => void;
  readonly onShown?: (page: Window | null) => void;
};

type Drawn = { html: string; turn: number };

export default function Page(props: Readonly<Props>) {
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
    let dressedUp: Worn = { lent: [], kept: [] };
    let live = true;
    const alive = () => live;
    const retheme = () => {
      if (held.contentDocument) dressed(held.contentDocument);
    };
    const loaded = () => {
      const page = held.contentDocument;
      if (!page?.querySelector(".page")) return;
      wired(page, () => reach.current);
      page.querySelector(".page")?.removeAttribute("inert");
      dressed(page);
      fitted(page, reach.current.wide, reach.current.zoom);
      reach.current.onOutline?.(heads(page));
      reach.current.onShown?.(held.contentWindow);
      dressedUp = worn(page, stranger, alive);
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
      for (const one of dressedUp.lent) one.drop();
      for (const one of dressedUp.kept) one();
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
