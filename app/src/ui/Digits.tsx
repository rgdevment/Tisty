import { useEffect, useRef } from "react";
import { fill, t } from "../locales";

export const HOW_MANY = 6;

const BOXES = ["one", "two", "three", "four", "five", "six"];

type Props = {
  label: string;
  value: string;
  onChange: (said: string) => void;
  onDone?: () => void;
};

export default function Digits({ label, value, onChange, onDone }: Props) {
  const boxes = useRef<Array<HTMLInputElement | null>>([]);

  useEffect(() => {
    boxes.current[0]?.focus();
  }, []);

  const said = (now: string) => {
    const kept = now.replace(/\D/g, "").slice(0, HOW_MANY);
    onChange(kept);
    boxes.current[Math.min(kept.length, HOW_MANY - 1)]?.focus();
  };

  return (
    <fieldset className="mt-2 flex gap-2 border-0 p-0">
      <legend className="sr-only">{label}</legend>
      {BOXES.map((named, n) => (
        <input
          key={named}
          ref={(box) => {
            boxes.current[n] = box;
          }}
          type="text"
          inputMode="numeric"
          autoComplete="one-time-code"
          aria-label={fill("digitOf", String(n + 1), String(HOW_MANY))}
          value={value[n] ?? ""}
          onChange={(e) => said(value.slice(0, n) + e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Backspace") {
              e.preventDefault();
              // The box you are standing on is the one that gives its digit back, not whichever
              // was written last: the arrows can leave you anywhere along the six.
              said(value.slice(0, value[n] === undefined ? -1 : n));
            }
            if (e.key === "Enter" && value.length === HOW_MANY) {
              e.preventDefault();
              onDone?.();
            }
            if (e.key === "ArrowLeft" && n > 0) {
              e.preventDefault();
              boxes.current[n - 1]?.focus();
            }
            if (e.key === "ArrowRight" && n < HOW_MANY - 1) {
              e.preventDefault();
              boxes.current[n + 1]?.focus();
            }
          }}
          onPaste={(e) => {
            e.preventDefault();
            // Nothing in the clipboard that could be a number is nothing to write: pasting the
            // wrong thing by accident must not wipe what has already been typed.
            const kept = e.clipboardData.getData("text").replace(/\D/g, "");
            if (kept) said(kept);
          }}
          className="h-11 w-9 rounded-[10px] border border-line bg-bg text-center font-mono text-[21px] text-ink caret-transparent outline-none focus:border-accent"
        />
      ))}
      <span className="sr-only" aria-live="polite">
        {value.length === HOW_MANY ? t("digitsFull") : ""}
      </span>
    </fieldset>
  );
}
