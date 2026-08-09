import { useEffect, useState } from "react";
import type { Counted, List } from "../core";
import { t } from "../locales";

export type Field = "date" | "deadline" | "list" | "tag" | "priority";

interface Props {
  from: Field | null;
  query: string;
  lists: List[];
  tags: Counted[];
  onDate: (field: "date" | "deadline") => void;
  onInsert: (marker: string) => void;
  onClose: () => void;
}

interface Row {
  key: string;
  glyph: string;
  label: string;
  say?: string;
  run: () => void;
}

export default function SlashMenu({ from, query, lists, tags, onDate, onInsert, onClose }: Props) {
  const [step, setStep] = useState<Field | null>(from);
  const [at, setAt] = useState(0);
  const rows = step === null ? fields(onDate, setStep) : within(step, lists, tags, onInsert);
  const shown = rows.filter((row) => fits(row, query));
  const on = Math.min(at, shown.length - 1);

  useEffect(() => setAt(0), [query, step]);
  useEffect(() => setStep(from), [from]);

  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") setAt((on + 1) % shown.length);
      else if (e.key === "ArrowUp") setAt((on - 1 + shown.length) % shown.length);
      else if (e.key === "Enter") shown[on]?.run();
      else if (e.key === "Escape") onClose();
      else return;
      e.preventDefault();
    };
    document.addEventListener("keydown", key, true);
    return () => document.removeEventListener("keydown", key, true);
  }, [shown, on, onClose]);

  if (shown.length === 0) return null;

  return (
    <div className="absolute top-1 left-0 z-20 w-[330px] rounded-[10px] border border-line bg-bg p-[5px] shadow-lg">
      {shown.map((row, i) => (
        <button
          key={row.key}
          type="button"
          onMouseEnter={() => setAt(i)}
          onClick={row.run}
          className={`flex w-full items-center gap-2.5 rounded-[7px] px-2.5 py-[7px] text-[13px] ${
            i === on ? "bg-accent-soft" : ""
          }`}
        >
          <span className="w-[15px] text-center text-[12px] text-soft">{row.glyph}</span>
          <span className="text-ink">{row.label}</span>
          {row.say && <span className="ml-auto text-[11px] text-faint">{row.say}</span>}
        </button>
      ))}
    </div>
  );
}

function fields(
  onDate: (field: "date" | "deadline") => void,
  setStep: (field: Field) => void,
): Row[] {
  return [
    {
      key: "date",
      glyph: "☀",
      label: t("fieldDate"),
      say: t("sayDate"),
      run: () => onDate("date"),
    },
    {
      key: "deadline",
      glyph: "⚑",
      label: t("fieldDeadline"),
      say: t("sayDeadline"),
      run: () => onDate("deadline"),
    },
    { key: "list", glyph: "@", label: t("fieldList"), say: "@", run: () => setStep("list") },
    { key: "tag", glyph: "#", label: t("fieldTag"), say: "#", run: () => setStep("tag") },
    {
      key: "priority",
      glyph: "!",
      label: t("fieldPriority"),
      say: t("sayPriority"),
      run: () => setStep("priority"),
    },
  ];
}

function within(
  step: Field,
  lists: List[],
  tags: Counted[],
  onInsert: (marker: string) => void,
): Row[] {
  if (step === "list") {
    return lists.map((list) => ({
      key: list.id,
      glyph: "@",
      label: list.name,
      run: () => onInsert(`@${slug(list.name)}`),
    }));
  }
  if (step === "tag") {
    return tags.map((counted) => ({
      key: counted.tag,
      glyph: "#",
      label: counted.tag,
      say: String(counted.tasks),
      run: () => onInsert(`#${counted.tag}`),
    }));
  }
  return ([1, 2, 3] as const).map((level) => {
    const label = t(level === 1 ? "high" : level === 2 ? "medium" : "low");
    return {
      key: String(level),
      glyph: "!",
      label,
      say: `!${level}`,
      run: () => onInsert(`!${label.toLowerCase()}`),
    };
  });
}

const fits = (row: Row, query: string): boolean =>
  query === "" || bare(row.label).startsWith(bare(query)) || row.key.startsWith(bare(query));

const bare = (text: string): string =>
  text
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "");

const slug = (name: string): string => name.toLowerCase().replace(/[ _]/g, "-");
