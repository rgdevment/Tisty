import { useEffect, useMemo, useRef, useState } from "react";
import { type Axis, banded, monthly, shelved } from "../archive";
import type { List, Task } from "../core";
import { cadence, isOverdue, stamped, whenLabel } from "../format";
import { fill, t } from "../locales";
import { edge, placed, said, tint } from "../quadrants";

interface Props {
  tasks: Task[];
  lists: List[];
  selected?: string;
  fresh?: string;
  reveal?: string;
  title: string;
  when?: string;
  count?: number;
  onBack?: () => void;
  bands?: "month" | "day";
  axis?: Axis;
  dense?: boolean;
  empty?: string;
  note?: string;
  onSelect: (id: string) => void;
  onComplete?: (id: string) => void;
  onFold?: (id: string, away: boolean) => void;
  onDrop?: (task: string, after?: string, before?: string) => void;
  above?: React.ReactNode;
  ask?: (id: string) => React.ReactNode;
  closing?: string;
  below?: React.ReactNode;
  instead?: React.ReactNode;
  children?: React.ReactNode;
}

export default function TaskList({
  tasks,
  lists,
  selected,
  fresh,
  reveal,
  title,
  when,
  count,
  onBack,
  bands,
  axis,
  dense,
  empty,
  note,
  onSelect,
  onComplete,
  onFold,
  above,
  ask,
  closing,
  below,
  instead,
  children,
}: Props) {
  const rows = useMemo(
    () =>
      axis && axis !== "time"
        ? shelved(tasks, axis, lists)
        : bands === "month"
          ? monthly(tasks)
          : bands === "day"
            ? banded(tasks)
            : tasks.map((task) => ({ kind: "one" as const, key: task.id, task, band: "" })),
    [tasks, bands, axis, lists],
  );
  const heads = useMemo(() => new Set(rows.map((row) => row.band)).size > 1, [rows]);
  const [shut, setShut] = useState<ReadonlySet<string>>(new Set());
  const hidden = (band: string) => heads && shut.has(band);
  const many = useMemo(() => {
    const tally = new Map<string, number>();
    for (const row of rows) tally.set(row.band, (tally.get(row.band) ?? 0) + 1);
    return tally;
  }, [rows]);

  const sheets = useMemo(() => {
    const out = new Map<string, typeof rows>();
    for (const row of rows) {
      const kept = out.get(row.band);
      if (kept) kept.push(row);
      else out.set(row.band, [row]);
    }
    return [...out].map(([band, held]) => ({ band, rows: held }));
  }, [rows]);

  const named = (id: string) => lists.find((list) => list.id === id)?.name;
  const columns = onFold
    ? "grid-cols-[20px_minmax(0,1fr)_auto_16px]"
    : "grid-cols-[20px_minmax(0,1fr)_auto]";
  const width = "mx-auto w-full max-w-[820px]";

  const asked = useRef<HTMLDivElement>(null);
  useEffect(() => {
    asked.current?.scrollIntoView?.({
      block: "nearest",
      behavior: window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth",
    });
  }, [reveal]);

  const listed = useRef<HTMLDivElement>(null);
  const [reached, setReached] = useState<string | null>(null);

  const drawn = useMemo(
    () =>
      sheets
        .flatMap((leaf) => leaf.rows)
        .filter((row) => !(heads && shut.has(row.band)))
        .map((row) => row.key),
    [sheets, shut, heads],
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

  const typed = (event: React.KeyboardEvent, task: Task, at: string) => {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      if (onComplete && task.status === "open") {
        walk(at, 1);
        onComplete(task.id);
        return;
      }
      if (onFold) {
        walk(at, 1);
        onFold(task.id, !task.hidden);
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
    walk(at, by);
  };

  const line = (task: Task, at: string) => {
    if (dense) {
      return (
        <div
          key={at}
          data-row={at}
          role="listitem"
          tabIndex={stops(at) ? 0 : -1}
          aria-label={task.status === "open" ? task.title : `${task.title} — ${t(task.status)}`}
          onFocus={() => setReached(at)}
          onKeyDown={(event) => typed(event, task, at)}
          onClick={() => onSelect(task.id)}
          className={`grid cursor-pointer grid-cols-[14px_minmax(0,1fr)_auto] items-baseline gap-2.5 rounded-md px-2.5 py-1 outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-accent ${
            selected === task.id ? "bg-active" : ""
          }`}
        >
          <span
            aria-hidden="true"
            className={`text-center text-[11.5px] ${
              task.status === "dropped" ? "text-faint" : "text-accent"
            }`}
          >
            {task.status === "dropped" ? "⨯" : "✓"}
          </span>
          <span className="truncate text-[13px] text-soft">{task.title}</span>
          <span className="text-[11.5px] whitespace-nowrap text-faint tabular-nums">
            {task.completed_at ? stamped(task.completed_at) : ""}
          </span>
        </div>
      );
    }
    return (
      <div key={at}>
        <div
          ref={reveal === task.id ? asked : undefined}
          data-row={at}
          role="listitem"
          tabIndex={stops(at) ? 0 : -1}
          aria-label={task.status === "open" ? task.title : `${task.title} — ${t(task.status)}`}
          aria-keyshortcuts={
            (onComplete && task.status === "open") || onFold ? "Control+Enter" : undefined
          }
          onFocus={() => setReached(at)}
          onKeyDown={(event) => typed(event, task, at)}
          onClick={() => onSelect(task.id)}
          className={`group grid cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-accent ${columns} items-start gap-2.5 rounded-[10px] px-2.5 py-2 hover:bg-hover ${
            selected === task.id ? "bg-active" : ""
          } ${fresh === task.id ? "bg-accent-soft transition-colors duration-700" : ""}`}
        >
          {onComplete && task.status === "open" ? (
            <button
              type="button"
              aria-label={fill("completeIt", task.title)}
              title={fill("completeIt", task.title)}
              tabIndex={-1}
              onClick={(e) => {
                e.stopPropagation();
                onComplete(task.id);
              }}
              className={`mt-0.5 h-4 w-4 rounded-full border-[1.5px] ${edge(task.priority)} ${
                closing === task.id ? "bg-accent" : ""
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
            <h2
              className={`text-[13px] leading-snug ${
                closing === task.id ? "text-faint line-through" : ""
              }`}
            >
              {task.title}
            </h2>
            <Meta task={task} list={task.list ? named(task.list) : undefined} />
          </div>

          <Volume task={task} />
          {onFold && (
            <button
              type="button"
              aria-label={task.hidden ? t("showIt") : t("hideIt")}
              title={task.hidden ? t("showIt") : t("hideIt")}
              tabIndex={-1}
              onKeyDown={(e) => e.stopPropagation()}
              onClick={(e) => {
                e.stopPropagation();
                onFold(task.id, !task.hidden);
              }}
              className="mt-0.5 flex h-4 w-4 items-center justify-center rounded-md text-[13px] leading-none text-faint opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100 hover:bg-line hover:text-ink"
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
        {onBack && (
          <button
            type="button"
            onClick={onBack}
            aria-label={t("goBack")}
            title={t("goBack")}
            className="-ml-5 shrink-0 self-center rounded-md px-1.5 py-0.5 text-[13px] text-faint hover:bg-hover hover:text-ink"
          >
            ‹
          </button>
        )}
        <div className="min-w-0 flex-1">
          <span className="flex items-baseline gap-2.5">
            <h1 className="text-[21px] font-semibold -tracking-[0.01em]">{title}</h1>
            <span className="text-[13px] tabular-nums text-faint">{count || ""}</span>
          </span>
          {when && (
            <span className="mt-0.5 block text-[12.5px] text-faint">
              <b className="font-semibold text-soft">{t("todayIs")}</b> · {when}
            </span>
          )}
        </div>
      </header>

      <div className={`shrink-0 px-5 pb-2 ${width}`}>{children}</div>
      {note && <p className={`shrink-0 px-7 pb-1.5 text-[11.5px] text-faint ${width}`}>{note}</p>}
      {above && <div className={`shrink-0 px-5 pb-1 ${width}`}>{above}</div>}

      <div
        ref={listed}
        role="list"
        aria-label={t("tasks")}
        className={`scroller flex flex-1 flex-col gap-3.5 px-5 pt-4 pb-6 ${width}`}
      >
        {instead}
        {!instead && tasks.length === 0 && (
          <p className="rounded-[10px] border border-hair bg-sheet px-5 py-4 text-[12.5px] leading-relaxed text-soft shadow-lift">
            {empty ?? t("nothingOpen")}
          </p>
        )}

        {!instead &&
          sheets.map((leaf) => (
            <section
              key={leaf.band || "all"}
              className="rounded-[10px] border border-hair bg-sheet px-2.5 py-2 shadow-lift"
            >
              {heads && leaf.band && (
                <button
                  type="button"
                  aria-expanded={!shut.has(leaf.band)}
                  onClick={() => setShut((was) => flip(was, leaf.band))}
                  className="mb-1 flex w-full items-baseline gap-2 border-b border-hair px-2.5 pb-1.5 text-left text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase hover:text-soft"
                >
                  <span aria-hidden="true" className="text-[9px]">
                    {shut.has(leaf.band) ? "▸" : "▾"}
                  </span>
                  {leaf.band}
                  <span className="ml-auto font-normal tracking-normal normal-case tabular-nums">
                    {many.get(leaf.band)}
                  </span>
                </button>
              )}

              {leaf.rows.map((row) => (
                <div key={row.key}>
                  {!hidden(row.band) && line(row.task, row.key)}
                  {!hidden(row.band) && ask?.(row.task.id)}
                </div>
              ))}
            </section>
          ))}

        {below}
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
  if (task.repeat) {
    bits.push(
      <span key="repeat" title={cadence(task.repeat)}>
        ↻ {cadence(task.repeat)}
      </span>,
    );
  }
  if (placed(task.priority)) {
    bits.push(
      <span key="priority" className={tint(task.priority)}>
        {said(task.priority)}
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
  return <div className="mt-0.5 flex flex-wrap gap-2.5 text-[11.5px] text-soft">{bits}</div>;
}

function Volume({ task }: { task: Task }) {
  const v = task.volume ?? {};

  const parts = [
    v.steps ? `${v.steps_done ?? 0}/${v.steps}` : null,
    v.journal ? `✎${v.journal}` : null,
  ].filter(Boolean);

  return (
    <span className="pt-px text-[11.5px] whitespace-nowrap text-faint">{parts.join(" · ")}</span>
  );
}

const flip = (was: ReadonlySet<string>, key: string): ReadonlySet<string> => {
  const next = new Set(was);
  if (!next.delete(key)) next.add(key);
  return next;
};
