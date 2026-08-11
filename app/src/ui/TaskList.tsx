import { useEffect, useMemo, useRef, useState } from "react";
import type { List, Task } from "../core";
import { cadence, isOverdue, whenLabel } from "../format";
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
  /// Said above the rows: what the list is not showing.
  note?: string;
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
  note,
  onSelect,
  onComplete,
  onFold,
  above,
  children,
}: Props) {
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
  const opens = useMemo(() => {
    const said = new Set<string>();
    return rows.map((row) => {
      if (!row.band || said.has(row.band)) return false;
      said.add(row.band);
      return true;
    });
  }, [rows]);

  const named = (id: string) => lists.find((list) => list.id === id)?.name;
  const columns = onFold
    ? "grid-cols-[20px_minmax(0,1fr)_auto_16px]"
    : "grid-cols-[20px_minmax(0,1fr)_auto]";
  const width = centred ? "mx-auto w-full max-w-[780px]" : "";

  // Never on capture: scrolling under someone who is still typing loses them.
  const asked = useRef<HTMLDivElement>(null);
  useEffect(() => {
    asked.current?.scrollIntoView({
      block: "nearest",
      behavior: window.matchMedia?.("(prefers-reduced-motion: reduce)").matches
        ? "auto"
        : "smooth",
    });
  }, [reveal]);

  // One stop for the whole list, not one per task: tabbing through ninety rows
  // to reach the sidebar is not navigation.
  const listed = useRef<HTMLDivElement>(null);
  const [reached, setReached] = useState<string | null>(null);

  // Only the rows that are actually drawn: a task inside a folded group is not
  // in the DOM, and one that was completed is not in the list any more. Anchor
  // the stop on either and the whole list becomes unreachable by keyboard.
  const drawn = useMemo(
    () =>
      rows.flatMap((row) =>
        row.kind === "one"
          ? [row.task.id]
          : open.has(row.key)
            ? row.tasks.map((one) => one.id)
            : [],
      ),
    [rows, open],
  );
  const anchor = reached !== null && drawn.includes(reached) ? reached : drawn[0];
  const stops = (id: string) => anchor === id;

  const walk = (from: string, by: number) => {
    const rows = Array.from(listed.current?.querySelectorAll<HTMLElement>("[data-row]") ?? []);
    const now = rows.findIndex((row) => row.dataset.row === from);
    const next = rows[now + by];
    if (!next) return;
    setReached(next.dataset.row ?? null);
    next.focus();
  };

  const typed = (event: React.KeyboardEvent, task: Task) => {
    // The verb this product exists for, and it was mouse-only: the circle is
    // out of the tab order on purpose, so without this there was no way at all.
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      if (onComplete && task.status === "open") {
        walk(task.id, 1);
        onComplete(task.id);
      }
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelect(task.id);
      return;
    }
    const by = event.key === "ArrowDown" ? 1 : event.key === "ArrowUp" ? -1 : 0;
    if (by === 0) return;
    event.preventDefault();
    walk(task.id, by);
  };


  const line = (task: Task) => {
    return (
      <div key={task.id}>
          <div
            ref={reveal === task.id ? asked : undefined}
            data-row={task.id}
            role="listitem"
            tabIndex={stops(task.id) ? 0 : -1}
            aria-label={task.title}
            aria-keyshortcuts={onComplete && task.status === "open" ? "Control+Enter" : undefined}
            onFocus={() => setReached(task.id)}
            onKeyDown={(event) => typed(event, task)}
            onClick={() => onSelect(task.id)}
            className={`group grid cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-accent ${columns} items-start gap-2.5 rounded-lg px-2.5 py-2 hover:bg-hover ${
              selected === task.id ? "bg-active" : ""
            } ${fresh === task.id ? "bg-accent-soft transition-colors duration-700" : ""}`}
          >
            {onComplete && task.status === "open" ? (
              <button
                aria-label={fill("completeIt", task.title)}
                title={fill("completeIt", task.title)}
                tabIndex={-1}
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
                tabIndex={-1}
                onKeyDown={(e) => e.stopPropagation()}
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
      {note && (
        <p className={`shrink-0 px-7 pb-1.5 text-[11.5px] text-faint ${width}`}>{note}</p>
      )}
      {above && <div className={`shrink-0 px-5 ${width}`}>{above}</div>}

      <div ref={listed} role="list" className={`scroller flex-1 px-5 pb-6 ${width}`}>
        {tasks.length === 0 && (
          <p className="px-2.5 py-4 text-sm leading-relaxed text-soft">
            {empty ?? t("nothingOpen")}
          </p>
        )}

        {rows.map((row, r) => (
          <div key={row.key}>
            {heads && opens[r] && (
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
  if (task.repeat) {
    bits.push(
      <span key="repeat" title={cadence(task.repeat)}>
        ↻ {cadence(task.repeat)}
      </span>,
    );
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
