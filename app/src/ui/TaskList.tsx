import { useEffect, useMemo, useRef, useState } from "react";
import type { List, Task } from "../core";
import { TASK } from "../drag";
import { isOverdue, whenLabel } from "../format";
import { banded, grouped } from "../archive";
import { fill, t } from "../locales";

interface Props {
  tasks: Task[];
  lists: List[];
  selected?: string;
  fresh?: string;
  reveal?: string;
  title: string;
  centred: boolean;
  bands?: "month" | "day";
  empty?: string;
  onSelect: (id: string) => void;
  onComplete?: (id: string) => void;
  onFold?: (id: string, away: boolean) => void;
  /** Undefined where the order is not the user's to set — the archive, a search. */
  onDrop?: (task: string, after?: string, before?: string) => void;
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
  bands,
  empty,
  onSelect,
  onComplete,
  onFold,
  onDrop,
  above,
  children,
}: Props) {
  const [under, setUnder] = useState<string | null>(null);
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  const rows = useMemo(
    () =>
      bands === "month"
        ? grouped(tasks)
        : bands === "day"
          ? banded(tasks)
          : tasks.map((task) => ({ kind: "one" as const, key: task.id, task, band: "" })),
    [tasks, bands],
  );
  // One heading over the whole list says nothing and costs a line: «Someday»
  // above a list where nothing is dated is noise, not structure.
  const heads = useMemo(() => new Set(rows.map((row) => row.band)).size > 1, [rows]);
  // `indexOf` per row turns the render quadratic on a long archive.
  const at = useMemo(() => new Map(tasks.map((task, i) => [task.id, i])), [tasks]);
  const [held, setHeld] = useState<string | null>(null);

  // `dragend` fires on the row that started it, and that row can be gone.
  useEffect(() => {
    if (held === null) return;
    const done = () => {
      setUnder(null);
      setHeld(null);
    };
    window.addEventListener("dragend", done);
    window.addEventListener("drop", done);
    return () => {
      window.removeEventListener("dragend", done);
      window.removeEventListener("drop", done);
    };
  }, [held]);

  // The core sorts by date, then priority, then the manual key. A drop across
  // either would snap back, so it is refused instead of promised.
  const settles = (moved?: Task, onto?: Task) =>
    moved !== undefined && onto !== undefined && group(moved) === group(onto);

  const lands = (i: number) =>
    onDrop && {
      draggable: true,
      onDragStart: (e: React.DragEvent) => {
        e.dataTransfer.setData(TASK, tasks[i].id);
        e.dataTransfer.effectAllowed = "move";
        setHeld(tasks[i].id);
      },
      onDragOver: (e: React.DragEvent) => {
        if (!e.dataTransfer.types.includes(TASK)) return;
        const moved = tasks.find((one) => one.id === held);
        if (!settles(moved, tasks[i])) {
          setUnder(null);
          return;
        }
        e.preventDefault();
        setUnder(tasks[i].id);
      },
      onDragLeave: () => setUnder((on) => (on === tasks[i].id ? null : on)),
      onDrop: (e: React.DragEvent) => {
        e.preventDefault();
        const id = e.dataTransfer.getData(TASK);
        setUnder(null);
        setHeld(null);
        if (!id || id === tasks[i].id) return;
        // Again here, not only on hover: a drop can arrive without one.
        if (!settles(tasks.find((one) => one.id === id), tasks[i])) return;
        // Dropped ON a row means «take its place», so it lands just above it.
        const above = tasks[i - 1]?.id === id ? tasks[i - 2] : tasks[i - 1];
        onDrop(id, settles(tasks[i], above) ? above?.id : undefined, tasks[i].id);
      },
    };
  const named = (id: string) => lists.find((list) => list.id === id)?.name;
  const columns = onFold
    ? "grid-cols-[20px_minmax(0,1fr)_auto_16px]"
    : "grid-cols-[20px_minmax(0,1fr)_auto]";
  const width = centred ? "mx-auto w-full max-w-[780px]" : "";

  // Never on capture: scrolling under someone who is still typing loses them.
  const asked = useRef<HTMLDivElement>(null);
  useEffect(() => {
    asked.current?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [reveal]);


  const line = (task: Task) => {
    const i = at.get(task.id) ?? -1;
    return (
      <div key={task.id}>
          <div
            ref={reveal === task.id ? asked : undefined}
            onClick={() => onSelect(task.id)}
            {...lands(i)}
            className={`group grid cursor-pointer ${columns} items-start gap-2.5 rounded-lg px-2.5 py-2 transition-colors duration-700 hover:bg-hover ${
              selected === task.id ? "bg-active" : ""
            } ${fresh === task.id ? "bg-accent-soft" : ""} ${
              under === task.id ? "border-t-2 border-accent" : "border-t-2 border-transparent"
            }`}
          >
            {onComplete && task.status === "open" ? (
              <button
                aria-label={task.title}
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
            ) : (
              <span
                title={task.status === "dropped" ? t("dropped") : t("done")}
                className={`mt-px text-[13px] ${task.status === "dropped" ? "text-faint" : "text-accent"}`}
              >
                {task.status === "dropped" ? "⨯" : "✓"}
              </span>
            )}

            <div className="min-w-0">
              <h2 className="text-sm leading-snug">{task.title}</h2>
              <Meta task={task} list={task.list ? named(task.list) : undefined} />
            </div>

            <Volume task={task} />
            {onFold && (
              <button
                aria-label={task.hidden ? t("showIt") : t("hideIt")}
                title={task.hidden ? t("showIt") : t("hideIt")}
                onClick={(e) => {
                  e.stopPropagation();
                  onFold(task.id, !task.hidden);
                }}
                className="mt-0.5 flex h-4 w-4 items-center justify-center rounded text-[13px] leading-none text-faint opacity-0 group-hover:opacity-100 hover:bg-line hover:text-ink"
              >
                {task.hidden ? "⊕" : "⊖"}
              </button>
            )}
          </div>
      </div>
    );
  };

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
        {tasks.length === 0 && (
          <p className="px-2.5 py-4 text-sm leading-relaxed text-soft">
            {empty ?? t("nothingOpen")}
          </p>
        )}

        {rows.map((row, r) => (
          <div key={row.key}>
            {heads && row.band && row.band !== rows[r - 1]?.band && (
              <div className="mt-5 mb-1 px-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase first:mt-1">
                {row.band}
              </div>
            )}

            {row.kind === "many" ? (
              <>
                <button
                  onClick={() => setOpen((was) => flip(was, row.key))}
                  aria-expanded={open.has(row.key)}
                  className="flex w-full items-center gap-2.5 rounded-lg px-2.5 py-2 text-left hover:bg-hover"
                >
                  <span className="w-4 shrink-0 text-center text-[13px] text-accent">✓</span>
                  <span className="min-w-0 truncate text-sm">{row.title}</span>
                  <span className="text-xs text-faint">
                    {fill("timesThisMonth", String(row.tasks.length))}
                  </span>
                  <span className="ml-auto text-[9px] text-faint">
                    {open.has(row.key) ? "▲" : "▼"}
                  </span>
                </button>
                {open.has(row.key) && (
                  <div className="ml-4 border-l border-hair pl-1">
                    {row.tasks.map((one) => line(one))}
                  </div>
                )}
              </>
            ) : (
              line(row.task)
            )}
          </div>
        ))}
      </div>
    </main>
  );
}

const flip = (was: ReadonlySet<string>, key: string): ReadonlySet<string> => {
  const next = new Set(was);
  if (!next.delete(key)) next.add(key);
  return next;
};

/// Everything the sort decides before the manual key gets a say.
const group = (task: Task): string => `${task.date?.at.slice(0, 10) ?? ""}|${task.priority}`;

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
  const v = task.volume ?? {};

  const parts = [
    v.steps ? `${v.steps_done ?? 0}/${v.steps}` : null,
    v.journal ? `✎${v.journal}` : null,
  ].filter(Boolean);

  return (
    <span className="pt-px text-xs whitespace-nowrap text-faint">{parts.join(" · ")}</span>
  );
}
