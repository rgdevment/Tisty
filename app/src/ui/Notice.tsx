import { useEffect } from "react";
import type { List, Task } from "../core";
import { whenLabel } from "../format";
import { t } from "../locales";

interface Props {
  task: Task;
  lists: List[];
  onOpen: () => void;
  onDismiss: () => void;
}

const LINGERS = 6000;

export default function Notice({ task, lists, onOpen, onDismiss }: Props) {
  useEffect(() => {
    const timer = setTimeout(onDismiss, LINGERS);
    return () => clearTimeout(timer);
  }, [task.id, onDismiss]);

  const list = lists.find((candidate) => candidate.id === task.list)?.name;
  const said = [
    task.date && whenLabel(task.date),
    task.deadline && `⚑ ${whenLabel(task.deadline)}`,
    list && `#${list}`,
    ...(task.tags ?? []).map((tag) => `@${tag}`),
  ].filter(Boolean);

  return (
    <div className="pointer-events-none fixed top-[46px] right-4 z-40 w-[280px]">
      <div
        onClick={onOpen}
        className="arrive pointer-events-auto flex cursor-pointer items-start gap-2.5 rounded-[10px] border border-line bg-bg p-[11px_13px] shadow-lg"
      >
        <span className="text-[13px] leading-snug text-accent">✓</span>
        <div className="min-w-0 flex-1">
          <h3 className="truncate text-[13px] leading-snug font-medium">{task.title}</h3>
          {said.length > 0 && (
            <p className="mt-0.5 truncate text-[11.5px] text-faint">{said.join(" · ")}</p>
          )}
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDismiss();
          }}
          title={t("close")}
          className="text-[11px] text-faint hover:text-ink"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
