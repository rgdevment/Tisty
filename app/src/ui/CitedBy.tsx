import { useState } from "react";
import type { Citing } from "../citing";
import { fill, t } from "../locales";
import Section from "./Section";
import { CLOSED, useCited } from "./useCited";

interface Props {
  task: string;
  /** The task this one is a part of, already named above it, so it is not told twice. */
  whole?: string;
  onDoc?: (id: string) => void;
  onTask?: (id: string) => void;
}

// A task cited from hundreds of documents would bury its column; the rest wait behind one press.
const FIRST = 12;

const glyphOf = (one: Citing): string => {
  if (one.kind === "task") return CLOSED.has(one.state ?? "") ? "✓" : "○";
  return one.state === "archived" ? "▢" : "▤";
};

export default function CitedBy({ task, whole, onDoc, onTask }: Props) {
  const citing = useCited(task);
  const [unfolded, setUnfolded] = useState(false);
  const all: Citing[] = Array.isArray(citing) ? citing : [];
  const shown = all.filter((one) => one.id !== whole && one.id !== task);
  if (shown.length === 0) return null;
  const listed = unfolded ? shown : shown.slice(0, FIRST);

  return (
    <>
      <Section label={t("citedBy")} note={`${shown.length}`} />
      <ul className="flex flex-col gap-1.5">
        {listed.map((one) => (
          <li key={one.id}>
            <button
              type="button"
              onClick={() => (one.kind === "task" ? onTask?.(one.id) : onDoc?.(one.id))}
              className="grid w-full cursor-pointer grid-cols-[17px_minmax(0,1fr)] items-baseline gap-2.5 rounded-[10px] border border-hair px-2.5 py-2 text-left hover:border-line"
            >
              <span aria-hidden="true" className="text-center text-[11.5px] text-faint">
                {glyphOf(one)}
              </span>
              <span className="min-w-0">
                <span
                  className={`block truncate text-[12.5px] ${CLOSED.has(one.state ?? "") ? "text-faint" : "text-ink"}`}
                >
                  {one.title.trim() || t("untitledDoc")}
                </span>
                {one.under?.length ? (
                  <span className="block truncate text-[11.5px] text-faint">
                    {fill("citedUnder", one.under.map((head) => head.title).join(" · "))}
                  </span>
                ) : null}
              </span>
            </button>
          </li>
        ))}
      </ul>
      {listed.length < shown.length && (
        <button
          type="button"
          onClick={() => setUnfolded(true)}
          className="mt-1.5 cursor-pointer text-[12.5px] text-accent hover:underline"
        >
          {fill("citedSeeAll", String(shown.length))}
        </button>
      )}
    </>
  );
}
