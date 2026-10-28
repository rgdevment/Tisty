import { useEffect, useRef } from "react";

/** Words typed in place: Enter or leaving keeps them, Escape leaves them as they were. */
export default function Renaming({
  text,
  label,
  onDone,
  className,
}: Readonly<{
  text: string;
  label: string;
  onDone: (text: string | null) => void;
  className: string;
}>) {
  const field = useRef<HTMLInputElement>(null);
  // Escape ends the typing and then the field loses focus; only the first of the two counts.
  const done = useRef(false);
  const finish = (said: string | null) => {
    if (done.current) return;
    done.current = true;
    onDone(said);
  };
  useEffect(() => {
    field.current?.focus();
    field.current?.select();
  }, []);
  return (
    <input
      ref={field}
      aria-label={label}
      defaultValue={text}
      className={`nodrag nopan rounded border border-accent bg-sheet px-1.5 text-[13px] text-ink outline-none ${className}`}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Enter") finish(e.currentTarget.value);
        if (e.key === "Escape") finish(null);
      }}
      onBlur={(e) => finish(e.currentTarget.value)}
    />
  );
}
