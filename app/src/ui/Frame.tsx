import { useEffect, useLayoutEffect, useRef } from "react";
import { diagrams } from "./diagrams";
import { fitted, type Reaching, type Worn, wired, worn } from "./dressing";
import { dressed, shell } from "./paging";
import { type Sheet, sheeting } from "./sheeting";

type Props = {
  readonly html: string;
  readonly turn: number;
  readonly label: string;
  readonly stranger: boolean;
  readonly tall: boolean;
  readonly untitled?: boolean;
  readonly shaping?: boolean;
  readonly hidden: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly sheet?: Sheet;
  readonly reach: () => Reaching;
  readonly onReady: (frame: HTMLIFrameElement) => void;
};

// The words change in place, and what the last ones left on the sheet itself goes with them.
const freshen = (sheet: Element, html: string) => {
  for (const name of sheet.getAttributeNames()) if (name !== "class") sheet.removeAttribute(name);
  sheet.className = "page";
  sheet.innerHTML = html;
};

const grows = (held: HTMLIFrameElement, page: Document): (() => void) => {
  const sheet = page.querySelector<HTMLElement>(".page");
  if (!sheet) return () => {};
  const sized = () => {
    held.style.height = `${Math.ceil(sheet.getBoundingClientRect().height)}px`;
  };
  sized();
  // Only the page's own window hears its elements change size.
  const Seeing = page.defaultView?.ResizeObserver;
  if (!Seeing) return () => {};
  const seeing = new Seeing(sized);
  seeing.observe(sheet);
  return () => seeing.disconnect();
};

export default function Frame(props: Readonly<Props>) {
  const {
    html,
    turn,
    label,
    stranger,
    tall,
    hidden,
    wide,
    zoom,
    untitled = false,
    shaping = false,
  } = props;
  const shaped = useRef(shaping);
  shaped.current = shaping;
  const frame = useRef<HTMLIFrameElement>(null);
  const said = useRef(props);
  said.current = props;
  const laid = useRef(() => {});

  const swap = useRef((_: string) => {});

  useEffect(() => {
    const held = frame.current;
    if (!held) return;
    let dressedUp: Worn = { lent: [], kept: [] };
    let stopGrowing = () => {};
    let stopSheeting = () => {};
    let live = true;
    let drawing = 0;
    const first = said.current.html;
    const undressed = () => {
      for (const one of dressedUp.lent) one.drop();
      for (const one of dressedUp.kept) one();
      dressedUp = { lent: [], kept: [] };
    };
    const dress = (page: Document) => {
      drawing += 1;
      const mine = drawing;
      dressedUp = worn(page, stranger, () => live && drawing === mine, said.current.reach());
    };
    const retheme = () => {
      const page = held.contentDocument;
      if (!page) return;
      dressed(page);
      const mine = drawing;
      void diagrams(
        page,
        page.documentElement.classList.contains("dark"),
        () => live && drawing === mine,
      );
    };
    // A frame moved in the page loads its first words again, so whatever is newer is put back.
    const loaded = () => {
      const page = held.contentDocument;
      const sheet = page?.querySelector(".page");
      // A document already wired keeps its listeners, and wiring it twice would answer every click twice.
      if (!page || !sheet || page.documentElement.hasAttribute("data-wired")) return;
      page.documentElement.setAttribute("data-wired", "");
      undressed();
      stopGrowing();
      stopSheeting();
      if (said.current.html !== first) freshen(sheet, said.current.html);
      wired(page, () => said.current.reach());
      sheet.removeAttribute("inert");
      page.documentElement.classList.toggle("tall", tall);
      page.documentElement.classList.toggle("untitled", untitled);
      page.documentElement.classList.toggle("shaping", shaped.current);
      dressed(page);
      fitted(page, said.current.wide, said.current.zoom);
      dress(page);
      if (tall) stopGrowing = grows(held, page);
      const sheets = sheeting(page, () => said.current.sheet);
      stopSheeting = sheets.stop;
      laid.current = sheets.lay;
      said.current.onReady(held);
    };
    swap.current = (next) => {
      const page = held.contentDocument;
      const sheet = page?.querySelector(".page");
      if (!page?.documentElement.hasAttribute("data-wired") || !sheet) return;
      undressed();
      // The document is built once, so the language and look it was built with are brought up to date.
      page.documentElement.lang = document.documentElement.lang;
      dressed(page);
      freshen(sheet, next);
      dress(page);
      laid.current();
      said.current.onReady(held);
    };
    const watching = new MutationObserver(retheme);
    watching.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    held.addEventListener("load", loaded);
    held.srcdoc = shell(first);
    return () => {
      live = false;
      swap.current = () => {};
      stopGrowing();
      stopSheeting();
      laid.current = () => {};
      watching.disconnect();
      held.removeEventListener("load", loaded);
      undressed();
    };
  }, [stranger, tall, untitled]);

  // Every drawing is dressed anew before it is painted, even one drawn the same: what it leans on may have changed.
  useLayoutEffect(() => swap.current(html), [html, turn]);

  useEffect(() => {
    frame.current?.contentDocument?.documentElement.classList.toggle("shaping", shaping);
  }, [shaping]);

  useEffect(() => {
    const page = frame.current?.contentDocument;
    if (page?.querySelector(".page")) fitted(page, wide, zoom);
  }, [wide, zoom]);

  const { tall: sheetTall, x: sheetX, y: sheetY } = props.sheet ?? {};
  useEffect(() => laid.current(), [sheetTall, sheetX, sheetY]);

  const place = hidden ? " pointer-events-none invisible absolute inset-x-0 top-0" : "";
  const size = tall ? " h-40" : " min-h-0 flex-1";
  return (
    <iframe
      ref={frame}
      title={label}
      aria-hidden={hidden || undefined}
      tabIndex={hidden ? -1 : undefined}
      className={`block w-full border-0 bg-sheet${size}${place}`}
      referrerPolicy="no-referrer"
    />
  );
}
