import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { List } from "../core";
import { fill, t } from "../locales";
import Glyph from "./Glyph";

interface Props {
  lists: List[];
  chosen: string[];
  onChange: (lists: string[]) => void;
}

const WIDE = 224;

export const said = (lists: List[], chosen: string[]): string => {
  if (chosen.length === 0) return t("onlyIn");
  if (chosen.length === 1) {
    return lists.find((one) => one.id === chosen[0])?.name ?? t("onlyIn");
  }
  return fill("listsAre", String(chosen.length));
};

export default function Only({ lists, chosen, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const [at, setAt] = useState<{ left: number; top?: number; bottom?: number; room: number }>({
    left: 0,
    room: 256,
  });
  const box = useRef<HTMLDivElement>(null);
  const panel = useRef<HTMLFieldSetElement>(null);

  // The panel lives at the end of the page, so focus is carried into it and back.
  useEffect(() => {
    if (!open) return;
    panel.current?.querySelector("input")?.focus({ preventScroll: true });
    const inside = (target: EventTarget | null) =>
      box.current?.contains(target as Node) || panel.current?.contains(target as Node);
    const away = (e: MouseEvent) => {
      if (!inside(e.target)) setOpen(false);
    };
    // Only a scroll that moves the button leaves the panel adrift; the app's own scrolls elsewhere do not.
    const scrolled = (e: Event) => {
      const moved = e.target instanceof Node && e.target.contains(box.current);
      if (moved && !inside(e.target)) setOpen(false);
    };
    const left = (e: FocusEvent) => {
      if (e.relatedTarget && !inside(e.relatedTarget)) setOpen(false);
    };
    const key = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      setOpen(false);
      box.current?.querySelector("button")?.focus();
    };
    const shut = () => setOpen(false);
    document.addEventListener("mousedown", away);
    document.addEventListener("scroll", scrolled, true);
    document.addEventListener("keydown", key);
    document.addEventListener("focusout", left);
    window.addEventListener("resize", shut);
    return () => {
      document.removeEventListener("mousedown", away);
      document.removeEventListener("scroll", scrolled, true);
      document.removeEventListener("keydown", key);
      document.removeEventListener("focusout", left);
      window.removeEventListener("resize", shut);
    };
  }, [open]);

  useEffect(() => {
    if (lists.length === 0) setOpen(false);
  }, [lists.length]);

  if (lists.length === 0) return null;

  const some = chosen.length > 0;
  const toggle = (id: string) =>
    onChange(chosen.includes(id) ? chosen.filter((one) => one !== id) : [...chosen, id]);

  return (
    <div ref={box} className="relative">
      <button
        type="button"
        aria-expanded={open}
        aria-haspopup="true"
        aria-pressed={some}
        onClick={(e) => {
          const seen = e.currentTarget.getBoundingClientRect();
          const left = Math.max(8, Math.min(seen.left, window.innerWidth - WIDE - 8));
          const below = window.innerHeight - seen.bottom - 12;
          setAt(
            below < 160 && seen.top > below
              ? { left, bottom: window.innerHeight - seen.top + 4, room: seen.top - 12 }
              : { left, top: seen.bottom + 4, room: below },
          );
          setOpen((was) => !was);
        }}
        className={`flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-[11.5px] ${
          some ? "border-ink bg-ink text-bg" : "border-line text-faint hover:text-soft"
        }`}
      >
        {said(lists, chosen)}
        <span aria-hidden="true" className="opacity-70">
          ▾
        </span>
      </button>

      {open &&
        createPortal(
          <fieldset
            ref={panel}
            style={{
              left: at.left,
              top: at.top,
              bottom: at.bottom,
              maxHeight: at.room,
              width: WIDE,
            }}
            className="fixed z-[70] flex flex-col rounded-[10px] border border-hair bg-bg p-1.5 shadow-lift"
          >
            <legend className="sr-only">{t("onlyIn")}</legend>
            <div className="scroller min-h-0 flex-1">
              {lists.map((list) => (
                <label
                  key={list.id}
                  className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-[12.5px] text-soft hover:bg-hover"
                >
                  <input
                    type="checkbox"
                    checked={chosen.includes(list.id)}
                    onChange={() => toggle(list.id)}
                  />
                  <span aria-hidden="true" className="w-4 shrink-0 text-center">
                    {list.icon ? <Glyph name={list.icon} /> : "○"}
                  </span>
                  <span className="min-w-0 flex-1 truncate">{list.name}</span>
                </label>
              ))}
            </div>
            {some && (
              <button
                type="button"
                onClick={() => onChange([])}
                className="mt-1 w-full border-hair border-t px-2 pt-1.5 pb-0.5 text-left text-[11.5px] text-faint hover:text-ink"
              >
                {t("onlyClear")}
              </button>
            )}
          </fieldset>,
          document.body,
        )}
    </div>
  );
}
