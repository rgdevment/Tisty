import { useEffect, useLayoutEffect, useRef, useState } from "react";

export interface Choice {
  key: string;
  label: string;
  icon?: string;
  danger?: boolean;
  apart?: boolean;
  off?: boolean;
  into?: { label: string; choices: Choice[] };
  onPick?: () => void;
}

interface Props {
  at: { x: number; y: number };
  choices: Choice[];
  label: string;
  onClose: () => void;
}

export default function Menu({ at, choices, label, onClose }: Props) {
  const card = useRef<HTMLDivElement>(null);
  const [where, setWhere] = useState(at);
  const [deeper, setDeeper] = useState<{ label: string; choices: Choice[] } | null>(null);
  const showing = deeper ?? { label, choices };
  const live = showing.choices.filter((one) => !one.off);

  useLayoutEffect(() => {
    const box = card.current?.getBoundingClientRect();
    if (!box) return;
    setWhere({
      x: Math.max(6, Math.min(at.x, window.innerWidth - box.width - 6)),
      y: Math.max(6, Math.min(at.y, window.innerHeight - box.height - 6)),
    });
  }, [at, deeper]);

  useEffect(() => {
    const came = document.activeElement as HTMLElement | null;
    return () => came?.focus?.();
  }, []);

  useEffect(() => {
    const items = card.current?.querySelectorAll<HTMLElement>("[role=menuitem]");
    items?.[deeper ? 1 : 0]?.focus();
  }, [deeper]);

  useEffect(() => {
    const away = (e: MouseEvent) => {
      if (!card.current?.contains(e.target as Node)) onClose();
    };
    document.addEventListener("mousedown", away);
    return () => document.removeEventListener("mousedown", away);
  }, [onClose]);

  const walk = (from: HTMLElement, by: number) => {
    const all = Array.from(card.current?.querySelectorAll<HTMLElement>("[role=menuitem]") ?? []);
    const now = all.indexOf(from);
    all[(now + by + all.length) % all.length]?.focus();
  };

  return (
    <div
      ref={card}
      role="menu"
      aria-label={showing.label}
      style={{ left: where.x, top: where.y }}
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
      className="scroller fixed z-[70] max-h-[min(420px,80vh)] min-w-[210px] rounded-lg border border-hair bg-rail py-1 shadow-lg"
    >
      {deeper && (
        <button
          type="button"
          role="menuitem"
          onClick={() => setDeeper(null)}
          className="mb-1 flex w-full items-center gap-2.5 border-b border-hair px-3 py-1.5 text-left text-[12.5px] text-faint hover:bg-hover focus:bg-hover focus:outline-none"
        >
          <span aria-hidden className="w-3.5 shrink-0 text-center text-[11px]">
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
            onClick={() => {
              if (one.into) return setDeeper(one.into);
              onClose();
              one.onPick?.();
            }}
            className={`flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[12.5px] hover:bg-hover focus:bg-hover focus:outline-none ${
              one.danger ? "text-urgent" : "text-ink"
            }`}
          >
            <span aria-hidden className="w-3.5 shrink-0 text-center text-[11px] text-faint">
              {one.icon ?? ""}
            </span>
            <span className="truncate">{one.label}</span>
            {one.into && (
              <span aria-hidden className="ml-auto pl-3 text-[11px] text-faint">
                ›
              </span>
            )}
          </button>
        </div>
      ))}
    </div>
  );
}
