import { useEffect, useRef } from "react";
import { fitted, type Reaching, type Worn, wired, worn } from "./dressing";
import { dressed } from "./paging";
import type { Told } from "./printing";
import { type Sheet, sheeting } from "./sheeting";
import { cardOn } from "./stamped";

type Props = {
  readonly html: string;
  readonly label: string;
  readonly stranger: boolean;
  readonly tall: boolean;
  readonly untitled?: boolean;
  readonly shaping?: boolean;
  readonly hidden: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly sheet?: Sheet;
  readonly stamp?: readonly Told[];
  readonly reach: () => Reaching;
  readonly onReady: (frame: HTMLIFrameElement) => void;
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

  useEffect(() => {
    const held = frame.current;
    if (!held) return;
    let dressedUp: Worn = { lent: [], kept: [] };
    let stopGrowing = () => {};
    let stopSheeting = () => {};
    let live = true;
    const alive = () => live;
    const retheme = () => {
      if (held.contentDocument) dressed(held.contentDocument);
    };
    const loaded = () => {
      const page = held.contentDocument;
      if (!page?.querySelector(".page")) return;
      wired(page, said.current.reach);
      page.querySelector(".page")?.removeAttribute("inert");
      page.documentElement.classList.toggle("tall", tall);
      page.documentElement.classList.toggle("untitled", untitled);
      page.documentElement.classList.toggle("shaping", shaped.current);
      dressed(page);
      fitted(page, said.current.wide, said.current.zoom);
      dressedUp = worn(page, stranger, alive, said.current.reach());
      if (tall) stopGrowing = grows(held, page);
      cardOn(page, said.current.stamp);
      const sheets = sheeting(page, () => said.current.sheet);
      stopSheeting = sheets.stop;
      laid.current = sheets.lay;
      said.current.onReady(held);
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
      stopGrowing();
      stopSheeting();
      laid.current = () => {};
      watching.disconnect();
      held.removeEventListener("load", loaded);
      for (const one of dressedUp.lent) one.drop();
      for (const one of dressedUp.kept) one();
    };
  }, [html, stranger, tall, untitled]);

  useEffect(() => {
    frame.current?.contentDocument?.documentElement.classList.toggle("shaping", shaping);
  }, [shaping]);

  useEffect(() => {
    const page = frame.current?.contentDocument;
    if (page?.querySelector(".page")) fitted(page, wide, zoom);
  }, [wide, zoom]);

  const card = JSON.stringify(props.stamp ?? []);
  useEffect(() => {
    const page = frame.current?.contentDocument;
    if (!page?.querySelector(".page")) return;
    cardOn(page, JSON.parse(card));
    laid.current();
  }, [card]);

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
