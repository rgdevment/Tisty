import { useState } from "react";
import type { Span } from "../core";

interface Props {
  icon: string;
  value: string;
  hint: string;
  marks?: Mark[];
  onChange: (value: string) => void;
  onSubmit?: () => void;
}

export interface Mark {
  span: Span;
  offered: boolean;
  overruled?: boolean;
}

export default function Field({ icon, value, hint, marks, onChange, onSubmit }: Props) {
  const [shift, setShift] = useState(0);
  const painted = marks !== undefined;

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit?.();
      }}
      className="flex w-full items-center gap-2.5 rounded-[9px] border border-line bg-bg px-3 py-2.5 focus-within:border-accent focus-within:ring-[3px] focus-within:ring-accent-soft"
    >
      <span className="w-4 shrink-0 text-center text-[13px] text-faint">{icon}</span>
      <div className="relative min-w-0 flex-1">
        {painted && <Mirror text={value} marks={marks} shift={shift} />}
        <input
          autoFocus
          value={value}
          placeholder={hint}
          aria-label={hint}
          onChange={(e) => onChange(e.target.value)}
          onScroll={(e) => setShift(e.currentTarget.scrollLeft)}
          className={`relative w-full bg-transparent text-sm leading-5 outline-none placeholder:text-faint ${
            painted ? "text-transparent caret-ink selection:bg-accent-soft" : ""
          }`}
        />
      </div>
    </form>
  );
}

const TINT: Record<Span["mark"], string> = {
  date: "bg-mark-date",
  deadline: "bg-mark-deadline",
  list: "bg-mark-list",
  tag: "bg-mark-tag",
  priority: "bg-mark-priority",
  repeat: "bg-mark-repeat",
};

const DOTTED: Record<Span["mark"], string> = {
  date: "decoration-accent",
  deadline: "decoration-high",
  list: "decoration-accent",
  tag: "decoration-accent",
  priority: "decoration-urgent",
  repeat: "decoration-faint",
};

function paint({ span, offered, overruled }: Mark): string {
  if (overruled) return "text-faint line-through decoration-faint";
  if (offered) return "underline decoration-dashed underline-offset-[3px] decoration-faint";
  return span.certainty === "sure"
    ? `rounded-[3px] ${TINT[span.mark]}`
    : `underline decoration-dotted underline-offset-[3px] ${DOTTED[span.mark]}`;
}

function Mirror({ text, marks, shift }: { text: string; marks: Mark[]; shift: number }) {
  return (
    <div aria-hidden className="pointer-events-none absolute inset-0 overflow-hidden">
      <div
        className="whitespace-pre text-sm leading-5"
        style={{ transform: `translateX(${-shift}px)` }}
      >
        {cut(text, marks).map((run) =>
          run.mark === undefined ? (
            <span key={`${run.at}:${run.text}`}>{run.text}</span>
          ) : (
            <span key={`${run.at}:${run.text}`} className={paint(run.mark)}>
              {run.text}
            </span>
          ),
        )}
      </div>
    </div>
  );
}

interface Run {
  at: number;
  text: string;
  mark?: Mark;
}

function cut(text: string, marks: Mark[]): Run[] {
  const chars = Array.from(text);
  const runs: Run[] = [];
  let at = 0;

  for (const mark of [...marks].sort((a, b) => a.span.from - b.span.from)) {
    const { from, to } = mark.span;
    if (from < at || to > chars.length) continue;
    if (from > at) runs.push({ at, text: chars.slice(at, from).join("") });
    runs.push({ at: from, text: chars.slice(from, to).join(""), mark });
    at = to;
  }
  if (at < chars.length) runs.push({ at, text: chars.slice(at).join("") });
  return runs;
}
