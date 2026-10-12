import { createElement } from "react";
import { SHAPES, type Shaped } from "./shapes";

export function Glyph({ name, className = "h-4 w-4" }: { name: Shaped; className?: string }) {
  const more = name === "more";
  return (
    <svg
      viewBox="0 0 16 16"
      className={className}
      fill={more ? "currentColor" : "none"}
      stroke={more ? "none" : "currentColor"}
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {SHAPES[name].map((one) =>
        createElement(one.tag, { key: JSON.stringify(one.at), ...one.at }),
      )}
    </svg>
  );
}

export function Broken({ empty }: { empty?: boolean }) {
  return (
    <svg
      viewBox="0 0 36 36"
      className="mx-auto mb-3.5 h-9 w-9 text-faint"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M9 4h12l8 8v18a2 2 0 0 1-2 2H9a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z" />
      <path d="M21 4v8h8" />
      {!empty && <path d="M13 21h10M13 25h6M4 32 32 4" />}
    </svg>
  );
}

export const tool =
  "grid h-7 w-7 shrink-0 place-items-center rounded-md text-soft hover:bg-hover hover:text-ink";

export const button =
  "shrink-0 whitespace-nowrap rounded-md border border-line px-[11px] py-1 text-[12.5px] text-ink hover:bg-hover disabled:opacity-50";

export const tinted =
  "shrink-0 whitespace-nowrap rounded-md bg-accent-soft px-[11px] py-1 text-[12.5px] font-semibold text-accent hover:brightness-95 disabled:opacity-50";

export function Segments<T extends string>({
  value,
  options,
  onPick,
  wide,
}: {
  value: T;
  options: { key: T; label: string }[];
  onPick: (key: T) => void;
  wide?: boolean;
}) {
  return (
    <div className={`flex gap-px rounded-md bg-hover p-0.5 ${wide ? "w-full" : ""}`}>
      {options.map((one) => (
        <button
          type="button"
          key={one.key}
          aria-pressed={value === one.key}
          onClick={() => onPick(one.key)}
          className={`${wide ? "flex-1" : ""} whitespace-nowrap rounded-md px-2.5 py-[3px] text-[12.5px] ${
            value === one.key ? "bg-sheet text-ink shadow-[0_1px_2px_rgb(0_0_0/0.12)]" : "text-soft"
          }`}
        >
          {one.label}
        </button>
      ))}
    </div>
  );
}
