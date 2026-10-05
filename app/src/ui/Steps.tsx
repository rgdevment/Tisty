import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { STEP_AT_MOST, type Step } from "../core";
import { fill, t } from "../locales";

interface Props {
  steps: Step[];
  onWrite: (text: string, step?: string) => void;
  onMark: (step: string, done: boolean) => void;
  onDrop: (step: string) => void;
  onTurn?: (step: string) => void;
}

const COUNTED_FROM = STEP_AT_MOST - 20;

const oneLine = (text: string): string => text.replace(/\s*\n\s*/g, " ");

// By character, as the core counts: maxLength counts UTF-16 units and an emoji would cost two.
const counted = (text: string): number => Array.from(text).length;

const within = (text: string, most: number): string =>
  counted(text) > most ? Array.from(text).slice(0, most).join("") : text;

export default function Steps({ steps, onWrite, onMark, onDrop, onTurn }: Props) {
  const [adding, setAdding] = useState("");
  const put = () => {
    if (adding.trim()) {
      onWrite(adding);
      setAdding("");
    }
  };

  return (
    <>
      {steps.map((step) => (
        <div key={step.id}>
          <Line step={step} onWrite={onWrite} onMark={onMark} onDrop={onDrop} onTurn={onTurn} />
        </div>
      ))}

      <form
        onSubmit={(e) => {
          e.preventDefault();
          put();
        }}
        className="flex items-start gap-2.5 py-1"
      >
        <span className="mt-0.5 h-[15px] w-[15px] shrink-0 rounded-md border-[1.5px] border-dashed border-line" />
        <textarea
          rows={1}
          value={adding}
          placeholder={t("addStep")}
          aria-label={t("addStep")}
          onChange={(e) => setAdding(within(oneLine(e.target.value), STEP_AT_MOST))}
          onKeyDown={(e) => {
            if (e.nativeEvent.isComposing) return;
            if (e.key === "Enter") {
              e.preventDefault();
              put();
            }
          }}
          className="field-sizing-content min-w-0 flex-1 resize-none bg-transparent text-[13px] outline-none placeholder:text-faint"
        />
        <Left text={adding} />
      </form>
    </>
  );
}

function Line({ step, onWrite, onMark, onDrop, onTurn }: { step: Step } & Omit<Props, "steps">) {
  const [text, setText] = useState(step.text);
  const dropped = useRef(false);
  useEffect(() => setText(step.text), [step.id, step.text]);

  const keyed = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (e.key === "Enter") {
      e.preventDefault();
      e.currentTarget.blur();
    }
    if (e.key === "Escape") {
      dropped.current = true;
      e.currentTarget.blur();
    }
  };

  return (
    <div className="group flex items-start gap-2.5 py-1 text-[13px]">
      <button
        type="button"
        role="checkbox"
        aria-checked={step.done}
        aria-label={fill(step.done ? "unmarkStep" : "markStep", step.text)}
        onClick={() => onMark(step.id, !step.done)}
        className={`mt-0.5 h-[15px] w-[15px] shrink-0 rounded-md border-[1.5px] ${
          step.done ? "border-accent bg-accent" : "border-faint hover:border-accent"
        }`}
      />
      <textarea
        rows={1}
        value={text}
        aria-label={fill("editStep", step.text)}
        onChange={(e) =>
          setText(within(oneLine(e.target.value), Math.max(STEP_AT_MOST, counted(step.text))))
        }
        onBlur={() => {
          if (dropped.current) {
            dropped.current = false;
            setText(step.text);
            return;
          }
          const kept = text.trim();
          if (kept && kept !== step.text) onWrite(kept, step.id);
          else setText(step.text);
        }}
        onKeyDown={keyed}
        className={`field-sizing-content min-w-0 flex-1 resize-none rounded-md bg-transparent outline-none hover:bg-hover focus:bg-hover ${
          step.done ? "text-faint line-through" : ""
        }`}
      />
      {text !== step.text && <Left text={text} />}
      {onTurn && !step.done && (
        <button
          type="button"
          aria-label={`${t("turnIntoTask")}: ${step.text}`}
          title={t("turnIntoTask")}
          onClick={() => onTurn(step.id)}
          className={`mt-0.5 flex h-4 shrink-0 items-center justify-center rounded-md px-1 text-[11.5px] text-faint outline-none hover:bg-line hover:text-ink focus-visible:opacity-100 focus-visible:ring-2 focus-visible:ring-accent ${
            counted(step.text) >= STEP_AT_MOST ? "text-accent" : "opacity-0 group-hover:opacity-100"
          }`}
        >
          ↗
        </button>
      )}
      <button
        type="button"
        aria-label={`${t("remove")} ${step.text}`}
        onClick={() => onDrop(step.id)}
        className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-md text-faint opacity-0 outline-none group-hover:opacity-100 hover:bg-line hover:text-ink focus-visible:opacity-100 focus-visible:ring-2 focus-visible:ring-accent"
      >
        ×
      </button>
    </div>
  );
}

function Left({ text }: { text: string }) {
  if (counted(text) < COUNTED_FROM) return null;
  const left = STEP_AT_MOST - counted(text);
  return (
    <span
      role="status"
      aria-label={fill("stepLeft", String(Math.max(left, 0)))}
      className={`mt-0.5 shrink-0 text-[10.5px] tabular-nums ${left < 0 ? "text-hue-red" : "text-faint"}`}
    >
      {left}
    </span>
  );
}
