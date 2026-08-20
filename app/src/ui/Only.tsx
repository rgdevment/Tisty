import { useEffect, useRef, useState } from "react";
import type { List } from "../core";
import { fill, t } from "../locales";
import { drawn, useIcons } from "./Icons";

interface Props {
  lists: List[];
  chosen: string[];
  onChange: (lists: string[]) => void;
}

export const said = (lists: List[], chosen: string[]): string => {
  if (chosen.length === 0) return t("onlyIn");
  if (chosen.length === 1) {
    return lists.find((one) => one.id === chosen[0])?.name ?? t("onlyIn");
  }
  return fill("listsAre", String(chosen.length));
};

export default function Only({ lists, chosen, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const all = useIcons();

  useEffect(() => {
    if (!open) return;
    const away = (e: MouseEvent) => {
      if (!box.current?.contains(e.target as Node)) setOpen(false);
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", away);
    document.addEventListener("keydown", key);
    return () => {
      document.removeEventListener("mousedown", away);
      document.removeEventListener("keydown", key);
    };
  }, [open]);

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
        onClick={() => setOpen((was) => !was)}
        className={`flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-[11.5px] ${
          some ? "border-ink bg-ink text-bg" : "border-line text-faint hover:text-soft"
        }`}
      >
        {said(lists, chosen)}
        <span aria-hidden="true" className="opacity-70">
          ▾
        </span>
      </button>

      {open && (
        <fieldset className="absolute left-0 z-30 mt-1 w-56 rounded-[10px] border border-hair bg-bg p-1.5 shadow-lift">
          <legend className="sr-only">{t("onlyIn")}</legend>
          <div className="scroller max-h-64">
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
                  {drawn(all, list.icon) ?? "○"}
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
        </fieldset>
      )}
    </div>
  );
}
