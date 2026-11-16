import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { folded } from "../folding";
import Glyph from "./Glyph";

export interface Choice {
  key: string;
  label: string;
  icon?: string;
  glyph?: string;
  mark?: string;
  danger?: boolean;
  apart?: boolean;
  off?: boolean;
  hint?: string;
  keys?: string;
  code?: string;
  on?: boolean;
  into?: { label: string; choices: Choice[] };
  onPick?: () => void;
}

interface Props {
  /** `top` is where the line the menu answers begins, so a menu put above it never covers it; `right` is its right edge. */
  at: { x: number; y: number; top?: number; right?: number };
  choices: Choice[];
  label: string;
  /** Opened from the bottom of the screen: `at.y` is the edge the menu grows up from. */
  up?: boolean;
  keeps?: boolean;
  find?: string;
  /** A line under the choices on how the keys walk them. */
  foot?: string;
  /** Put away without a choice, it hands back what was typed in its search, so those words are not lost. */
  onClose: (sought?: string) => void;
}

const EDGE = 6;
const GAP = 4;
const TALLEST = 0.8;
const SHORTEST = 120;
const ROOMY_BELOW = 240;

// The bar above a page holds its title and its menu, so a menu flipped up stops under it.
const ceilingOver = (top: number) =>
  Array.from(document.querySelectorAll(".ribbon"), (one) => one.getBoundingClientRect().bottom)
    .filter((bottom) => bottom <= top)
    .reduce((most, bottom) => Math.max(most, bottom), EDGE);

export default function Menu({ at, choices, label, up, keeps, find, foot, onClose }: Props) {
  const card = useRef<HTMLDivElement>(null);
  const [where, setWhere] = useState<{ x: number; y: number; tall?: number }>(at);
  const [deeper, setDeeper] = useState<{ label: string; choices: Choice[] } | null>(null);
  const [asked, setAsked] = useState("");
  const typed = useRef("");
  const showing = deeper ?? { label, choices };
  const sought = deeper ? "" : folded(asked.trim());
  const live = showing.choices.filter(
    (one) => !one.off && folded(`${one.label} ${one.code ?? ""}`).includes(sought),
  );
  const sifting = Boolean(find) && !deeper;
  // Enter takes the first plain choice found; a list of its own opens only when nothing else is.
  const plain = live.findIndex((one) => !one.into);
  const leading = plain >= 0 ? plain : live.findIndex((one) => one.into);

  // Down when it fits below or there is fair room to scroll there, up over its anchor otherwise, never over the page's bar.
  useLayoutEffect(() => {
    const own = card.current;
    if (!own) return;
    const box = own.getBoundingClientRect();
    const edges = own.offsetHeight - own.clientHeight;
    const tall = Math.min(own.scrollHeight + edges, Math.round(window.innerHeight * TALLEST));
    const top = at.top ?? at.y;
    const ceiling = ceilingOver(top);
    const below = window.innerHeight - EDGE - at.y;
    const above = top - GAP - ceiling;
    const rightward = at.right !== undefined && at.x > window.innerWidth / 2;
    const left = rightward ? (at.right ?? at.x) - box.width : at.x;
    const x = Math.max(EDGE, Math.min(left, window.innerWidth - box.width - EDGE));
    const over = (high: number) => ({ x, y: top - GAP - high, tall: high });
    if (up && tall <= above) return setWhere(over(tall));
    if (tall <= below) return setWhere({ x, y: at.y, tall });
    if (below >= ROOMY_BELOW) return setWhere({ x, y: at.y, tall: below });
    if (tall <= above) return setWhere(over(tall));
    const roomy = Math.max(below, above);
    if (roomy < SHORTEST)
      return setWhere({ x, y: Math.max(EDGE, window.innerHeight - tall - EDGE), tall });
    setWhere(below >= above ? { x, y: at.y, tall: below } : over(above));
  }, [at, deeper, up, live.length]);

  useEffect(() => {
    const came = document.activeElement as HTMLElement | null;
    const own = card.current;
    // A choice that put the focus somewhere of its own, a field to type in, keeps it there.
    return () => {
      const now = document.activeElement;
      if (!now || now === document.body || own?.contains(now)) came?.focus?.();
    };
  }, []);

  useEffect(() => {
    if (sifting) return card.current?.querySelector<HTMLElement>("input")?.focus();
    const items = card.current?.querySelectorAll<HTMLElement>("[role=menuitem]");
    items?.[deeper ? 1 : 0]?.focus();
  }, [deeper, sifting]);

  useEffect(() => {
    const away = (e: Event) => {
      if (!card.current?.contains(e.target as Node)) onClose(typed.current);
    };
    // A click in a page's frame never reaches this document; it takes the focus from the window instead.
    const left = () => onClose(typed.current);
    const listen = setTimeout(() => {
      document.addEventListener("mousedown", away);
      document.addEventListener("scroll", away, true);
      window.addEventListener("blur", left);
    });
    return () => {
      clearTimeout(listen);
      document.removeEventListener("mousedown", away);
      document.removeEventListener("scroll", away, true);
      window.removeEventListener("blur", left);
    };
  }, [onClose]);

  const walk = (from: HTMLElement, by: number) => {
    const all = Array.from(card.current?.querySelectorAll<HTMLElement>("[role=menuitem]") ?? []);
    const now = all.indexOf(from);
    all[(now + by + all.length) % all.length]?.focus();
  };

  return createPortal(
    <div
      ref={card}
      role="menu"
      aria-label={showing.label}
      data-keeps-writing={keeps ? "" : undefined}
      style={{ left: where.x, top: where.y, maxHeight: where.tall }}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.stopPropagation();
          if (deeper) return setDeeper(null);
          onClose(typed.current);
        }
        if (e.key === "ArrowLeft" && deeper) {
          e.preventDefault();
          setDeeper(null);
        }
        if (e.key === "ArrowDown" || e.key === "ArrowUp") {
          e.preventDefault();
          walk(e.target as HTMLElement, e.key === "ArrowDown" ? 1 : -1);
        }
      }}
      className="scroller arrive fixed z-[70] max-h-[80vh] min-w-[210px] [scrollbar-gutter:stable] rounded-[10px] border border-hair bg-rail p-1 shadow-lg"
    >
      {sifting && (
        <input
          type="search"
          value={asked}
          placeholder={find}
          aria-label={find}
          onChange={(e) => {
            typed.current = e.target.value;
            // A space right after the slash is a sentence going on, as in «10 / 2», not a block being sought.
            if (e.target.value.startsWith(" ")) return onClose(e.target.value);
            setAsked(e.target.value);
          }}
          onKeyDown={(e) => {
            if (e.key !== "Enter") return;
            e.preventDefault();
            const first = live[leading];
            if (!first) return;
            if (first.into) return setDeeper(first.into);
            onClose();
            first.onPick?.();
          }}
          className="mx-1 mb-1 w-[calc(100%-0.5rem)] rounded-md bg-hover px-2.5 py-1.5 text-[12.5px] text-ink outline-none placeholder:text-faint"
        />
      )}
      {deeper && (
        <button
          type="button"
          role="menuitem"
          onClick={() => setDeeper(null)}
          className="mb-1 flex w-full items-center gap-2.5 border-b border-hair px-3 py-1.5 text-left text-[12.5px] text-faint hover:bg-hover focus:bg-hover focus:outline-none"
        >
          <span aria-hidden className="w-3.5 shrink-0 text-center text-[11.5px]">
            ‹
          </span>
          <span className="truncate">{deeper.label}</span>
        </button>
      )}
      {live.map((one, index) => (
        <div key={one.key}>
          {one.apart && <div className="my-1 h-px bg-hair" />}
          <button
            type="button"
            role="menuitem"
            aria-haspopup={one.into ? "menu" : undefined}
            title={one.hint}
            onClick={() => {
              if (one.into) return setDeeper(one.into);
              onClose();
              one.onPick?.();
            }}
            onMouseMove={(e) => {
              if (!sifting && document.activeElement !== e.currentTarget) e.currentTarget.focus();
            }}
            className={`group flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[12.5px] hover:bg-hover focus:bg-hover focus:outline-none ${
              sifting && sought && index === leading ? "bg-hover " : ""
            }${one.danger ? "text-urgent" : "text-ink"}${one.on ? " font-semibold" : ""}`}
          >
            {one.glyph ? (
              <Glyph
                name={one.glyph}
                className="h-[15px] w-[15px] text-faint group-hover:text-accent group-focus:text-accent"
              />
            ) : one.code ? (
              <span aria-hidden className="w-9 shrink-0 text-[10.5px] tracking-[0.04em] text-faint">
                {one.code}
              </span>
            ) : (
              <span aria-hidden className="w-3.5 shrink-0 text-center text-[11.5px] text-faint">
                {one.icon ?? ""}
              </span>
            )}
            <span className="truncate">{one.label}</span>
            {one.mark && (
              <span aria-hidden className="ml-auto pl-4 font-mono text-[10.5px] text-faint">
                {one.mark}
              </span>
            )}
            {one.keys && (
              <span aria-hidden className="ml-auto pl-4 text-[11.5px] text-faint">
                {one.keys}
              </span>
            )}
            {one.on && (
              <span aria-hidden className="ml-auto pl-3 text-[11.5px] text-faint">
                ✓
              </span>
            )}
            {one.into && (
              <span aria-hidden className="ml-auto pl-3 text-[11.5px] text-faint">
                ›
              </span>
            )}
          </button>
        </div>
      ))}
      {foot && (
        <p className="mx-1 mt-1 border-hair border-t px-2 pt-1.5 pb-0.5 text-[10.5px] leading-snug text-faint">
          {foot}
        </p>
      )}
    </div>,
    document.body,
  );
}
