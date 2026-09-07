import { useEffect, useRef } from "react";
import { fill, t } from "../locales";

export const HOW_MANY = 6;

type Props = {
  label: string;
  value: string;
  onChange: (said: string) => void;
  onDone?: () => void;
};

export default function Digits({ label, value, onChange, onDone }: Props) {
  const boxes = useRef<Array<HTMLInputElement | null>>([]);
  const at = Math.min(value.length, HOW_MANY - 1);

  useEffect(() => {
    boxes.current[0]?.focus();
  }, []);

  const said = (now: string) => {
    const kept = now.replace(/\D/g, "").slice(0, HOW_MANY);
    onChange(kept);
    boxes.current[Math.min(kept.length, HOW_MANY - 1)]?.focus();
    if (kept.length === HOW_MANY) onDone?.();
  };

  return (
    <div role="group" aria-label={label} className="mt-2 flex gap-2">
      {Array.from({ length: HOW_MANY }, (_, n) => (
        <input
          key={`digit-${n}`}
          ref={(box) => {
            boxes.current[n] = box;
          }}
          type="text"
          inputMode="numeric"
          autoComplete="one-time-code"
          aria-label={fill("digitOf", String(n + 1), String(HOW_MANY))}
          value={value[n] ?? ""}
          onFocus={() => boxes.current[at]?.focus()}
          onChange={(e) => said(value.slice(0, n) + e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Backspace") {
              e.preventDefault();
              said(value.slice(0, -1));
            }
          }}
          onPaste={(e) => {
            e.preventDefault();
            said(e.clipboardData.getData("text"));
          }}
          className="h-11 w-9 rounded-[9px] border border-line bg-bg text-center font-mono text-[17px] text-ink caret-transparent outline-none focus:border-accent"
        />
      ))}
      <span className="sr-only" aria-live="polite">
        {value.length === HOW_MANY ? t("digitsFull") : ""}
      </span>
    </div>
  );
}
