import { useEffect, useRef } from "react";
import type { List, Task } from "../core";
import { isOverdue, whenLabel } from "../format";
import { t } from "../locales";

interface Props {
  tasks: Task[];
  lists: List[];
  selected?: string;
  fresh?: string;
  reveal?: string;
  title: string;
  centred: boolean;
  onSelect: (id: string) => void;
  onComplete: (id: string) => void;
  above?: React.ReactNode;
  children?: React.ReactNode;
}

export default function TaskList({
  tasks,
  lists,
  selected,
  fresh,
  reveal,
  title,
  centred,
  onSelect,
  onComplete,
  above,
  children,
}: Props) {
  const named = (id: string) => lists.find((list) => list.id === id)?.name;
  const width = centred ? "mx-auto w-full max-w-[780px]" : "";

  // Never on capture: scrolling under someone who is still typing loses them.
  const asked = useRef<HTMLDivElement>(null);
  useEffect(() => {
    asked.current?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [reveal]);

  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <header className={`flex items-baseline gap-2.5 px-8 pb-3.5 ${width}`}>
        <h1 className="text-[21px] font-semibold -tracking-[0.01em]">{title}</h1>
        <span className="text-[13px] text-faint">{tasks.length || ""}</span>
      </header>

      <div className={`shrink-0 px-5 pb-2 ${width}`}>{children}</div>
      {above && <div className={`shrink-0 px-5 ${width}`}>{above}</div>}

      <div className={`scroller flex-1 px-5 pb-6 ${width}`}>
        {tasks.length === 0 && <p className="px-2.5 py-4 text-sm text-faint">{t("nothingOpen")}</p>}

        {tasks.map((task) => (
          <div
            key={task.id}
            ref={reveal === task.id ? asked : undefined}
            onClick={() => onSelect(task.id)}
            className={`grid cursor-pointer grid-cols-[20px_1fr_auto] items-start gap-2.5 rounded-lg px-2.5 py-2 transition-colors duration-700 hover:bg-hover ${
              selected === task.id ? "bg-active" : ""
            } ${fresh === task.id ? "bg-accent-soft" : ""}`}
          >
            <button
              onClick={(e) => {
                e.stopPropagation();
                onComplete(task.id);
              }}
              className={`mt-0.5 h-4 w-4 rounded-full border-[1.5px] ${
                task.priority === 1
                  ? "border-urgent"
                  : task.priority === 2
                    ? "border-high"
                    : "border-faint"
              }`}
            />

            <div className="min-w-0">
              <h2 className="text-sm leading-snug">{task.title}</h2>
              <Meta task={task} list={task.list ? named(task.list) : undefined} />
            </div>

            <Volume task={task} />
          </div>
        ))}
      </div>
    </main>
  );
}

function Meta({ task, list }: { task: Task; list?: string }) {
  const bits: React.ReactNode[] = [];

  if (task.date) {
    bits.push(
      <span key="date" className={isOverdue(task.date) ? "text-urgent" : "text-accent"}>
        {whenLabel(task.date)}
      </span>,
    );
  }
  if (task.deadline) {
    bits.push(<span key="deadline">⚑ {whenLabel(task.deadline)}</span>);
  }
  if (task.priority === 1 || task.priority === 2) {
    bits.push(
      <span key="priority" className={task.priority === 1 ? "text-urgent" : "text-high"}>
        {t(task.priority === 1 ? "high" : "medium")}
      </span>,
    );
  }
  if (list) bits.push(<span key="list">@{list}</span>);
  if (task.tags?.length) {
    bits.push(
      <span key="tags" className="text-faint">
        {task.tags.map((tag) => `#${tag}`).join(" ")}
      </span>,
    );
  }

  if (bits.length === 0) return null;
  return <div className="mt-0.5 flex flex-wrap gap-2.5 text-xs text-soft">{bits}</div>;
}

function Volume({ task }: { task: Task }) {
  const v = task.volume;
  if (!v) return null;

  const parts = [
    v.steps ? `${v.steps_done ?? 0}/${v.steps}` : null,
    v.journal ? `✎${v.journal}` : null,
  ].filter(Boolean);

  if (parts.length === 0) return null;
  return <span className="pt-px text-xs whitespace-nowrap text-faint">{parts.join(" · ")}</span>;
}
