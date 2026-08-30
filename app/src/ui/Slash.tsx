import { useEffect, useRef } from "react";
import { matched } from "../finding";
import { t } from "../locales";
import Glyph, { known } from "./Glyph";

export const asked = (before: string): string | null => {
  const found = /(?:^|\s)\/([\p{L}\p{N}_]*)$/u.exec(before);
  return found ? found[1] : null;
};

export const narrowed = <T extends { label: string }>(blocks: T[], word: string): T[] =>
  word ? blocks.filter((one) => matched(one.label, word)) : blocks;

export interface Block {
  key: string;
  label: string;
  hint: string;
  icon: string;
  run: () => void;
}

interface Props {
  at: { x: number; y: number };
  blocks: Block[];
  active: number;
  onPick: (block: Block) => void;
}

export default function Slash({ at, blocks, active, onPick }: Props) {
  const card = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const one = card.current?.querySelectorAll<HTMLElement>("[data-block]")[active];
    one?.scrollIntoView?.({ block: "nearest" });
  }, [active]);

  const room = Math.min(at.y + 6, window.innerHeight - 210);

  return (
    <div
      ref={card}
      id="slash-menu"
      role="listbox"
      aria-label="/"
      style={{ left: Math.min(at.x, window.innerWidth - 232), top: Math.max(8, room) }}
      className="scroller fixed z-[70] max-h-[204px] w-[212px] rounded-[10px] border border-hair bg-rail p-1 shadow-xl"
    >
      {blocks.length === 0 && (
        <p className="px-2.5 py-1.5 text-[12.5px] text-faint">{t("noneHere")}</p>
      )}
      {blocks.map((one, i) => (
        <button
          key={one.key}
          id={`slash-${i}`}
          type="button"
          data-block
          role="option"
          tabIndex={-1}
          aria-selected={i === active}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => onPick(one)}
          className={`flex w-full items-center gap-2 rounded-md px-2 py-[5px] text-left ${
            i === active ? "bg-accent-soft" : ""
          }`}
        >
          <span
            aria-hidden
            className={`flex w-4 shrink-0 justify-center text-[11px] ${
              i === active ? "text-accent" : "text-faint"
            }`}
          >
            {known(one.icon) ? <Glyph name={one.icon} className="h-[15px] w-[15px]" /> : one.icon}
          </span>
          <span className="min-w-0 flex-1 truncate text-[12.5px] text-ink">{one.label}</span>
          <span
            aria-hidden
            className="shrink-0 font-mono text-[10px] text-faint tabular-nums opacity-70"
          >
            {one.hint}
          </span>
        </button>
      ))}
    </div>
  );
}
