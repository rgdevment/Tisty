import { useEffect, useRef, useState } from "react";
import { t } from "../locales";
import { docCard } from "../markdown";
import Calendar from "./Calendar";
import Papers from "./Papers";

/** The cells written by picking rather than typing. */
export type Picked = "date" | "select" | "tags" | "link";

type Props = {
  readonly kind: Picked;
  /** The cell's left edge, its foot, and its top, to open above it when there is no room below. */
  readonly at: { x: number; y: number; above: number };
  /** The cell as the file has it. */
  readonly was: string;
  /** The cell as it reads, marks aside, which is how an option or a tag is told. */
  readonly shown: string;
  /** The values the column already holds, for an option or a tag. */
  readonly options: string[];
  readonly onWrite: (said: string) => void;
  readonly onClose: () => void;
};

const ISO = /^\d{4}-\d{2}-\d{2}$/;

const tagsOf = (said: string): string[] =>
  said
    .split(",")
    .map((one) => one.trim())
    .filter(Boolean);

// One field to type in what the list does not offer; Enter writes it.
function Typed({ label, onTyped }: { label: string; onTyped: (said: string) => void }) {
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => field.current?.focus(), []);
  return (
    <input
      ref={field}
      aria-label={label}
      placeholder={label}
      className="w-full rounded-md border border-line bg-sheet px-2 py-1 text-[13px] outline-none focus:border-accent"
      onKeyDown={(event) => {
        if (event.key !== "Enter") return;
        event.preventDefault();
        const said = event.currentTarget.value.trim();
        if (said) onTyped(said);
        event.currentTarget.value = "";
      }}
    />
  );
}

const option =
  "rounded-full border border-line px-2.5 py-0.5 text-[12.5px] hover:bg-hover aria-pressed:border-accent aria-pressed:bg-accent-soft";

// About as tall as the calendar or a list grows, to know whether it fits under the cell.
const TALL = { date: 380, select: 260, tags: 260, link: 360 };

export default function TableEditor({ kind, at, was, shown, options, onWrite, onClose }: Props) {
  const [tags, setTags] = useState(() => tagsOf(shown));
  const tagsNow = useRef(tags);
  tagsNow.current = tags;
  const wide = kind === "date" ? 288 : 272;
  const below = at.y + 4 + TALL[kind] <= window.innerHeight;
  const placed = {
    width: `${wide}px`,
    left: Math.max(8, Math.min(at.x, window.innerWidth - wide - 18)),
    ...(below ? { top: at.y + 4 } : { bottom: Math.max(8, window.innerHeight - at.above + 4) }),
  };
  // Tags are chosen a few at a time and written once, when the list is left.
  const close = () => {
    const now = tagsNow.current.join(", ");
    if (kind === "tags" && now !== tagsOf(shown).join(", ")) onWrite(now);
    else onClose();
  };
  const toggled = (one: string) =>
    setTags((all) =>
      all.some((was) => was.toLowerCase() === one.toLowerCase())
        ? all.filter((was) => was.toLowerCase() !== one.toLowerCase())
        : [...all, one],
    );
  const offered = kind === "tags" ? [...new Set([...options, ...tags])] : options;

  return (
    <>
      <span aria-hidden="true" className="fixed inset-0 z-30" onMouseDown={close} />
      <div
        role="dialog"
        aria-label={t("table")}
        style={placed}
        onKeyDown={(event) => {
          if (event.key !== "Escape") return;
          event.preventDefault();
          event.stopPropagation();
          kind === "tags" ? close() : onClose();
        }}
        className="fixed z-40 flex max-h-[420px] flex-col gap-2 overflow-auto rounded-[10px] border border-hair bg-rail p-2 shadow-xl"
      >
        {kind === "date" && (
          <>
            <Typed label={t("tableDateSaid")} onTyped={onWrite} />
            <Calendar
              inline
              value={ISO.test(was) ? was : undefined}
              onPick={onWrite}
              onClear={() => onWrite("")}
              onClose={onClose}
            />
          </>
        )}
        {(kind === "select" || kind === "tags") && (
          <>
            <div className="flex flex-wrap gap-1.5">
              {offered.map((one) => (
                <button
                  key={one}
                  type="button"
                  aria-pressed={
                    kind === "tags"
                      ? tags.some((was) => was.toLowerCase() === one.toLowerCase())
                      : one.toLowerCase() === shown.toLowerCase()
                  }
                  className={option}
                  onClick={() => (kind === "tags" ? toggled(one) : onWrite(one))}
                >
                  {one}
                </button>
              ))}
            </div>
            <Typed
              label={kind === "tags" ? t("tableTagNew") : t("tableOptionNew")}
              onTyped={(said) => (kind === "tags" ? toggled(said) : onWrite(said))}
            />
            {kind === "select" && was && (
              <button type="button" className={`${option} self-start`} onClick={() => onWrite("")}>
                {t("tableClear")}
              </button>
            )}
          </>
        )}
        {kind === "link" && (
          <>
            <span className="px-1 text-[11.5px] text-faint">{t("tableDocPick")}</span>
            <Papers onPick={(doc) => onWrite(docCard(doc.file, doc.title || t("untitledDoc")))} />
          </>
        )}
      </div>
    </>
  );
}
