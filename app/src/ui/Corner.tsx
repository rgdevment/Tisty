import type { ReactNode } from "react";
import { useEffect } from "react";

interface Act {
  label: string;
  onClick: () => void;
}

interface Props {
  apart: string;
  label: string;
  mark: ReactNode;
  title: string;
  why: string;
  go: Act;
  no: Act;
  later: Act;
}

export default function Corner({ apart, label, mark, title, why, go, no, later }: Props) {
  useEffect(() => {
    const away = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.stopPropagation();
      later.onClick();
    };
    window.addEventListener("keydown", away, true);
    return () => window.removeEventListener("keydown", away, true);
  }, [later.onClick]);

  return (
    <section
      role="status"
      aria-label={label}
      className={`arrive absolute bottom-3 z-30 w-[280px] rounded-[10px] border border-line bg-bg px-[13px] py-3 shadow-lift ${apart}`}
    >
      <div className="flex items-start gap-2.5">
        {mark}
        <span className="min-w-0 flex-1">
          <b className="block text-[13px] leading-snug font-semibold">{title}</b>
          <span className="mt-1 block text-[11.5px] leading-relaxed text-faint">{why}</span>
        </span>
        <button
          type="button"
          aria-label={later.label}
          title={later.label}
          onClick={later.onClick}
          className="-mr-1 shrink-0 cursor-pointer rounded-md px-1 text-[11.5px] text-faint outline-none hover:text-ink focus-visible:ring-2 focus-visible:ring-accent"
        >
          ✕
        </button>
      </div>
      <div className="mt-3 flex flex-wrap items-center justify-between gap-2">
        <button
          type="button"
          onClick={go.onClick}
          className="cursor-pointer rounded-[10px] bg-accent px-3 py-1.5 text-[12.5px] font-medium text-bg outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-bg"
        >
          {go.label}
        </button>
        <button
          type="button"
          onClick={no.onClick}
          className="cursor-pointer rounded-[10px] border border-line px-2.5 py-1 text-[12.5px] text-soft outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-accent"
        >
          {no.label}
        </button>
      </div>
    </section>
  );
}
