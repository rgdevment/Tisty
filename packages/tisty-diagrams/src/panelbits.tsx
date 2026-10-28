import type { ReactNode } from "react";

export const field =
  "rounded-md border border-line bg-desk px-2.5 py-1.5 text-[13px] text-ink outline-none focus:border-accent";
export const caption = "text-[12px] font-semibold text-soft";

// Words are kept as typed and given to the diagram when the field is left, so a half-written name never renames.
export function Said({
  id,
  label,
  value,
  onSay,
}: Readonly<{ id: string; label: string; value: string; onSay: (said: string) => void }>) {
  return (
    <>
      <label htmlFor={id} className={caption}>
        {label}
      </label>
      <input
        id={id}
        key={value}
        defaultValue={value}
        className={field}
        onBlur={(e) => {
          if (e.currentTarget.value !== value) onSay(e.currentTarget.value);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") e.currentTarget.blur();
        }}
      />
    </>
  );
}

export const Ticked = ({
  label,
  on,
  onFlip,
}: Readonly<{ label: string; on: boolean; onFlip: () => void }>) => (
  <label className="flex items-center gap-1.5 text-[12.5px] text-soft">
    <input type="checkbox" checked={on} onChange={onFlip} />
    {label}
  </label>
);

export const Section = ({ title, children }: Readonly<{ title: string; children: ReactNode }>) => (
  <div className="flex flex-col gap-2 border-b border-hair pb-4">
    <span className="text-[12px] font-bold tracking-wide text-faint uppercase">{title}</span>
    {children}
  </div>
);

/** A choice among a few, shown as a native select so the keyboard and screen readers know it. */
export function Chosen<T extends string>({
  id,
  label,
  value,
  options,
  onChoose,
}: Readonly<{
  id: string;
  label: string;
  value: T;
  options: readonly { value: T; label: string }[];
  onChoose: (value: T) => void;
}>) {
  return (
    <>
      <label htmlFor={id} className={caption}>
        {label}
      </label>
      <select
        id={id}
        className={field}
        value={value}
        onChange={(e) => onChoose(e.target.value as T)}
      >
        {options.map((one) => (
          <option key={one.value} value={one.value}>
            {one.label}
          </option>
        ))}
      </select>
    </>
  );
}
