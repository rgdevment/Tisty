import { useEffect, useRef, useState } from "react";
import { STEP } from "../drag";
import type { Step } from "../core";
import { t } from "../locales";

interface Props {
  steps: Step[];
  onWrite: (text: string, step?: string) => void;
  onMark: (step: string, done: boolean) => void;
  onDrop: (step: string) => void;
  onMove: (step: string, after?: string, before?: string) => void;
}

export default function Steps({ steps, onWrite, onMark, onDrop, onMove }: Props) {
  const [adding, setAdding] = useState("");
  const [under, setUnder] = useState<string | null>(null);

  const lands = (i: number) => ({
    onDragOver: (e: React.DragEvent) => {
      if (!e.dataTransfer.types.includes(STEP)) return;
      e.preventDefault();
      setUnder(steps[i].id);
    },
    onDragLeave: () => setUnder((on) => (on === steps[i].id ? null : on)),
    onDrop: (e: React.DragEvent) => {
      e.preventDefault();
      const moved = e.dataTransfer.getData(STEP);
      setUnder(null);
      if (moved && moved !== steps[i].id) onMove(moved, steps[i - 1]?.id, steps[i].id);
    },
  });

  return (
    <>
      {steps.map((step, i) => (
        <div key={step.id} {...lands(i)}>
          <Line
            step={step}
            under={under === step.id}
            onWrite={onWrite}
            onMark={onMark}
            onDrop={onDrop}
          />
        </div>
      ))}

      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (adding.trim()) {
            onWrite(adding);
            setAdding("");
          }
        }}
        className="flex items-center gap-2.5 py-1"
      >
        <span className="h-[15px] w-[15px] shrink-0 rounded border-[1.5px] border-dashed border-line" />
        <input
          value={adding}
          placeholder={t("addStep")}
          aria-label={t("addStep")}
          onChange={(e) => setAdding(e.target.value)}
          className="min-w-0 flex-1 bg-transparent text-[13.5px] outline-none placeholder:text-faint"
        />
      </form>
    </>
  );
}

function Line({
  step,
  under,
  onWrite,
  onMark,
  onDrop,
}: { step: Step; under: boolean } & Omit<Props, "steps" | "onMove">) {
  const [text, setText] = useState(step.text);
  const dropped = useRef(false);
  useEffect(() => setText(step.text), [step.id, step.text]);

  return (
    <div
      draggable
      onDragStart={(e) => {
        e.dataTransfer.setData(STEP, step.id);
        e.dataTransfer.effectAllowed = "move";
      }}
      className={`group flex items-start gap-2.5 border-t-2 py-1 text-[13.5px] ${
        under ? "border-accent" : "border-transparent"
      }`}
    >
      <button
        type="button"
        aria-label={step.text}
        onClick={() => onMark(step.id, !step.done)}
        className={`mt-0.5 h-[15px] w-[15px] shrink-0 rounded border-[1.5px] ${
          step.done ? "border-accent bg-accent" : "border-faint hover:border-accent"
        }`}
      />
      <input
        value={text}
        aria-label={step.text}
        onChange={(e) => setText(e.target.value)}
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
        onKeyDown={(e) => {
          if (e.key === "Enter") e.currentTarget.blur();
          if (e.key === "Escape") {
            dropped.current = true;
            e.currentTarget.blur();
          }
        }}
        className={`min-w-0 flex-1 rounded bg-transparent outline-none hover:bg-hover focus:bg-hover ${
          step.done ? "text-faint line-through" : ""
        }`}
      />
      <button
        type="button"
        aria-label={`${t("remove")} ${step.text}`}
        onClick={() => onDrop(step.id)}
        className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded text-faint opacity-0 group-hover:opacity-100 hover:bg-line hover:text-ink"
      >
        ×
      </button>
    </div>
  );
}
