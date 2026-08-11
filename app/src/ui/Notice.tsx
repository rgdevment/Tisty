import { useEffect, useState } from "react";
import type { List, Task } from "../core";
import { whenLabel } from "../format";
import { fill, t } from "../locales";

interface Props {
  /// True when what was just filed does not show in the list behind this card:
  /// written from «upcoming» or «repeating», a task with no date lands on today
  /// and vanishes from view the instant it is typed.
  elsewhere?: boolean;
  task: Task;
  lists: List[];
  onOpen: () => void;
  onDismiss: () => void;
}

const LINGERS = 6000;

export default function Notice({ task, lists, elsewhere, onOpen, onDismiss }: Props) {
  // Six seconds is plenty to read and nowhere near enough to tab to. It only
  // runs out while nobody is holding the card.
  const [reading, setReading] = useState(false);

  useEffect(() => {
    if (reading) return;
    const timer = setTimeout(onDismiss, LINGERS);
    return () => clearTimeout(timer);
  }, [task.id, reading, onDismiss]);

  const list = lists.find((candidate) => candidate.id === task.list)?.name;
  const said = [
    task.date && whenLabel(task.date),
    task.deadline && `⚑ ${whenLabel(task.deadline)}`,
    list && `@${list}`,
    ...(task.tags ?? []).map((tag) => `#${tag}`),
  ].filter(Boolean);

  return (
    <div className="pointer-events-none fixed top-[46px] right-4 z-40 w-[280px]">
      <div
        role="status"
        onFocus={() => setReading(true)}
        onBlur={(e) => {
          if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setReading(false);
        }}
        className="arrive pointer-events-auto flex items-start gap-2.5 rounded-[10px] border border-line bg-bg p-[11px_13px] shadow-lift"
      >
        <button
          type="button"
          onClick={onOpen}
          aria-label={fill("openIt", task.title)}
          className="flex min-w-0 flex-1 items-start gap-2.5 rounded text-left outline-none focus-visible:ring-2 focus-visible:ring-accent"
        >
          <span aria-hidden="true" className="text-[13px] leading-snug text-accent">
            ✓
          </span>
          <span className="min-w-0 flex-1">
            <span className="block truncate text-[13px] leading-snug font-medium">{task.title}</span>
            {said.length > 0 && (
              <span className="mt-0.5 block truncate text-[11.5px] text-faint">
                {said.join(" · ")}
              </span>
            )}
            {elsewhere && (
              <span className="mt-0.5 block text-[11.5px] text-accent">{t("filedElsewhere")}</span>
            )}
          </span>
        </button>
        <button
          type="button"
          onClick={onDismiss}
          aria-label={t("close")}
          title={t("close")}
          className="rounded text-[11px] text-faint outline-none hover:text-ink focus-visible:ring-2 focus-visible:ring-accent"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
