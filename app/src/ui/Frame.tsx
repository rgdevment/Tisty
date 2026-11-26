import { useEffect, useLayoutEffect, useRef } from "react";
import { type Design, diagrams, fontWorn, type Sheet, sheeting } from "tisty-editor/read";
import { caught } from "../broke";
import { unnavigated } from "../unnavigated";
import { fitted, type Reaching, type Worn, wired, worn } from "./dressing";
import { dressedAsWindow, framed } from "./framed";
import { onWindows } from "./macKeys";
import { wornLaid } from "./pageDressing";
import { followTheme } from "./theming";

type Props = {
  readonly html: string;
  readonly turn: number;
  readonly label: string;
  readonly tall: boolean;
  readonly untitled?: boolean;
  readonly viewing?: boolean;
  readonly shaping?: boolean;
  /** An agent's file, drawn inside its card with no paper of its own. */
  readonly agent?: boolean;
  readonly hidden: boolean;
  readonly wide?: number;
  readonly zoom?: number;
  readonly sheet?: Sheet;
  readonly design?: Design;
  readonly reach: () => Reaching;
  readonly onReady: (frame: HTMLIFrameElement) => void;
  /** Hands over how blocks drawn again in place are dressed, as the page they join was. */
  readonly onDresser?: (dress: (blocks: readonly Element[]) => void) => void;
};

// Windows keeps arrows on even the thin bar, so the viewer draws its own there.
const ARROWED = onWindows;

// The words change in place, and what the last ones left on the sheet itself goes with them.
const freshen = (sheet: Element, html: string) => {
  for (const name of sheet.getAttributeNames()) if (name !== "class") sheet.removeAttribute(name);
  sheet.className = "page";
  sheet.innerHTML = html;
};

const steady = (step: () => void) => {
  try {
    step();
  } catch (why) {
    caught(why);
  }
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
    tall,
    hidden,
    wide,
    zoom,
    untitled = false,
    viewing = false,
    shaping = false,
    agent = false,
  } = props;
  const shaped = useRef(shaping);
  shaped.current = shaping;
  const carded = useRef(agent);
  carded.current = agent;
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
      const was = dressedUp;
      dressedUp = { lent: [], kept: [] };
      for (const one of was.lent) steady(() => one.drop());
      for (const one of was.kept) steady(one);
    };
    const dress = (page: Document) => {
      drawing += 1;
      const mine = drawing;
      dressedUp = worn(page, () => live && drawing === mine, said.current.reach());
    };
    const retheme = () => {
      const page = held.contentDocument;
      if (!page?.documentElement) return;
      dressedAsWindow(page, said.current.design);
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
      page.addEventListener("keydown", unnavigated);
      undressed();
      steady(stopGrowing);
      steady(stopSheeting);
      stopGrowing = () => {};
      stopSheeting = () => {};
      laid.current = () => {};
      if (said.current.html !== first) freshen(sheet, said.current.html);
      sheet.removeAttribute("inert");
      page.documentElement.classList.toggle("tall", tall);
      page.documentElement.classList.toggle("untitled", untitled);
      page.documentElement.classList.toggle("own-bar", viewing && ARROWED);
      page.documentElement.classList.toggle("shaping", shaped.current);
      page.documentElement.classList.toggle("agent", carded.current);
      steady(() => wired(page, () => said.current.reach()));
      steady(() => dressedAsWindow(page, said.current.design));
      steady(() => fitted(page, said.current.wide, said.current.zoom));
      steady(() => dress(page));
      steady(() => {
        if (tall) stopGrowing = grows(held, page);
      });
      steady(() => {
        const sheets = sheeting(page, () => said.current.sheet);
        stopSheeting = sheets.stop;
        laid.current = sheets.lay;
      });
      steady(() =>
        said.current.onDresser?.((blocks) => {
          const mine = drawing;
          const kept = wornLaid(page, blocks, () => live && drawing === mine, said.current.reach());
          dressedUp.kept.push(...kept);
          laid.current();
        }),
      );
      steady(() => said.current.onReady(held));
      void fontWorn(page, said.current.design ?? "tisty").then(() => live && laid.current());
    };
    swap.current = (next) => {
      const page = held.contentDocument;
      const sheet = page?.querySelector(".page");
      if (!page?.documentElement?.hasAttribute("data-wired") || !sheet) return;
      undressed();
      // The document is built once, so the language and look it was built with are brought up to date.
      page.documentElement.lang = document.documentElement.lang;
      steady(() => dressedAsWindow(page, said.current.design));
      freshen(sheet, next);
      steady(() => dress(page));
      steady(() => laid.current());
      steady(() => said.current.onReady(held));
    };
    const unfollow = followTheme(retheme);
    held.addEventListener("load", loaded);
    held.srcdoc = framed(first, said.current.reach().bar?.css);
    return () => {
      live = false;
      swap.current = () => {};
      steady(stopGrowing);
      steady(stopSheeting);
      laid.current = () => {};
      unfollow();
      held.removeEventListener("load", loaded);
      undressed();
    };
  }, [tall, untitled, viewing]);

  // Every drawing is dressed anew before it is painted, even one drawn the same: what it leans on may have changed.
  useLayoutEffect(() => swap.current(html), [html, turn]);

  useEffect(() => {
    frame.current?.contentDocument?.documentElement?.classList.toggle("shaping", shaping);
  }, [shaping]);

  useEffect(() => {
    frame.current?.contentDocument?.documentElement?.classList.toggle("agent", agent);
  }, [agent]);

  useEffect(() => {
    const page = frame.current?.contentDocument;
    if (page?.querySelector(".page")) fitted(page, wide, zoom);
  }, [wide, zoom]);

  const { design } = props;
  useLayoutEffect(() => {
    const page = frame.current?.contentDocument;
    if (page?.documentElement?.dataset.wired === undefined) return;
    dressedAsWindow(page, design);
    laid.current();
    let gone = false;
    void fontWorn(page, design ?? "tisty").then(() => {
      if (!gone) laid.current();
    });
    return () => {
      gone = true;
    };
  }, [design]);

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
