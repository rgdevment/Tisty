const PATHS: Record<string, React.ReactNode> = {
  list: <path d="M3 4.5h10M3 8h7M3 11.5h10" />,
  reload: (
    <>
      <path d="M13.2 8.6A5.3 5.3 0 1 1 11.8 4" />
      <path d="M12.7 1.8v3h-3" />
    </>
  ),
  apart: (
    <>
      <path d="M9.5 2.5H13.5V6.5M13.5 2.5L8 8" />
      <path d="M6.5 3.5H3.5A1 1 0 0 0 2.5 4.5V12.5A1 1 0 0 0 3.5 13.5H11.5A1 1 0 0 0 12.5 12.5V9.5" />
    </>
  ),
  code: <path d="M5.5 4.5 2 8l3.5 3.5M10.5 4.5 14 8l-3.5 3.5" />,
  lock: (
    <>
      <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" />
      <path d="M5.5 7V5.2a2.5 2.5 0 0 1 5 0V7" />
    </>
  ),
  doc: (
    <>
      <path d="M4 1.8h5l3.5 3.5v9a.9.9 0 0 1-.9.9H4a.9.9 0 0 1-.9-.9V2.7A.9.9 0 0 1 4 1.8Z" />
      <path d="M9 1.8v3.6h3.5M5.5 8.5h5M5.5 11h3.5" />
    </>
  ),
  widget: <path d="M8 2 14 8 8 14 2 8Z" />,
};

export function Glyph({ name, className = "h-4 w-4" }: { name: string; className?: string }) {
  if (name === "more")
    return (
      <svg viewBox="0 0 16 16" className={className} fill="currentColor" aria-hidden="true">
        <circle cx="3.5" cy="8" r="1.3" />
        <circle cx="8" cy="8" r="1.3" />
        <circle cx="12.5" cy="8" r="1.3" />
      </svg>
    );
  return (
    <svg
      viewBox="0 0 16 16"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {PATHS[name]}
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
