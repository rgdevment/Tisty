import { useEffect, useRef } from "react";

/// The word being typed after a slash, or null when there is no menu to show.
/// A slash mid-word is a path or a date, not a command.
export const asked = (before: string): string | null => {
  const found = /(?:^|\s)\/([\p{L}\p{N}_]*)$/u.exec(before);
  return found ? found[1] : null;
};

export const narrowed = <T extends { label: string }>(blocks: T[], word: string): T[] =>
  word ? blocks.filter((one) => one.label.toLowerCase().includes(word.toLowerCase())) : blocks;

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

  const room = Math.min(at.y + 8, window.innerHeight - 260);

  return (
    <div
      ref={card}
      role="listbox"
      aria-label="/"
      style={{ left: Math.min(at.x, window.innerWidth - 260), top: Math.max(8, room) }}
      className="scroller fixed z-[70] max-h-[248px] w-[240px] rounded-lg border border-hair bg-rail py-1 shadow-lg"
    >
      {blocks.map((one, i) => (
        <button
          key={one.key}
          type="button"
          data-block
          role="option"
          aria-selected={i === active}
          onMouseDown={(e) => {
            e.preventDefault();
            onPick(one);
          }}
          className={`flex w-full items-center gap-2.5 px-3 py-1.5 text-left ${
            i === active ? "bg-hover" : ""
          }`}
        >
          <span aria-hidden className="w-4 shrink-0 text-center text-[12px] text-faint">
            {one.icon}
          </span>
          <span className="min-w-0">
            <span className="block truncate text-[12.5px] text-ink">{one.label}</span>
            <span className="block truncate text-[11px] text-faint">{one.hint}</span>
          </span>
        </button>
      ))}
    </div>
  );
}
