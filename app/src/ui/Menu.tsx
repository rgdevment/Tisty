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
  at: { x: number; y: number };
  choices: Choice[];
  label: string;
  /** Opened from the bottom of the screen: `at.y` is the edge the menu grows up from. */
  up?: boolean;
  keeps?: boolean;
  find?: string;
  /** A line under the choices on how the keys walk them. */
  foot?: string;
  onClose: () => void;
}

const EDGE = 6;
const TALLEST = 0.8;

export default function Menu({ at, choices, label, up, keeps, find, foot, onClose }: Props) {
  const card = useRef<HTMLDivElement>(null);
  const [where, setWhere] = useState<{ x: number; y: number; tall?: number }>(at);
  const [deeper, setDeeper] = useState<{ label: string; choices: Choice[] } | null>(null);
  const [asked, setAsked] = useState("");
  const showing = deeper ?? { label, choices };
  const sought = deeper ? "" : folded(asked.trim());
  const live = showing.choices.filter(
    (one) => !one.off && folded(`${one.label} ${one.code ?? ""}`).includes(sought),
  );
  const sifting = Boolean(find) && !deeper;

  // Down when it fits below, up when it fits above, and only past what the window holds does it scroll.
  useLayoutEffect(() => {
    const own = card.current;
    if (!own) return;
    const box = own.getBoundingClientRect();
    const room = window.innerHeight - EDGE * 2;
    const edges = own.offsetHeight - own.clientHeight;
    const tall = Math.min(own.scrollHeight + edges, Math.round(window.innerHeight * TALLEST));
    const below = at.y + tall <= window.innerHeight - EDGE;
    const above = at.y - tall >= EDGE;
    const y = (up || !below) && above ? at.y - tall : at.y;
    setWhere({
      x: Math.max(EDGE, Math.min(at.x, window.innerWidth - box.width - EDGE)),
      y: Math.max(EDGE, Math.min(y, window.innerHeight - tall - EDGE)),
      tall: Math.min(tall, room),
    });
  }, [at, deeper, up]);

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
      if (!card.current?.contains(e.target as Node)) onClose();
    };
    // A click in a page's frame never reaches this document; it takes the focus from the window instead.
    const left = () => onClose();
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
          onClose();
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
      className="scroller arrive fixed z-[70] max-h-[80vh] min-w-[210px] rounded-[10px] border border-hair bg-rail p-1 shadow-lg"
    >
      {sifting && (
        <input
          type="search"
          value={asked}
          placeholder={find}
          aria-label={find}
          onChange={(e) => setAsked(e.target.value)}
          onKeyDown={(e) => {
            if (e.key !== "Enter") return;
            e.preventDefault();
            const first = live.find((one) => !one.into);
            if (!first) return;
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
      {live.map((one) => (
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
            className={`group flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-left text-[12.5px] hover:bg-hover focus:bg-hover focus:outline-none ${
              one.danger ? "text-urgent" : "text-ink"
            }${one.on ? " font-semibold" : ""}`}
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
