import type { List, Task } from "../core";
import { whenLabel } from "../format";
import { t } from "../locales";

interface Props {
  task: Task;
  lists: List[];
  expanded: boolean;
  onExpand: () => void;
  onCollapse: () => void;
}

export default function Detail({ task, lists, expanded, onExpand, onCollapse }: Props) {
  const list = lists.find((l) => l.id === task.list);

  const body = (
    <>
      <h2
        className={`leading-snug font-semibold -tracking-[0.01em] ${expanded ? "text-[22px]" : "text-[17px]"} mb-3`}
      >
        {task.title}
      </h2>

      <div className="mb-5 flex flex-wrap gap-1.5">
        {list && <Chip>▤ {list.name}</Chip>}
        {task.date && <Chip tone="accent">☀ {whenLabel(task.date)}</Chip>}
        {task.deadline && <Chip>⚑ {whenLabel(task.deadline)}</Chip>}
        {task.priority < 4 && (
          <Chip tone={task.priority === 1 ? "urgent" : undefined}>
            {t(task.priority === 1 ? "urgent" : task.priority === 2 ? "high" : "medium")}
          </Chip>
        )}
        {task.tags?.map((tag) => <Chip key={tag}>#{tag}</Chip>)}
        {task.reminders?.length ? <Chip>⏰ {task.reminders.length}</Chip> : null}
      </div>

      {task.description && <p className="text-[13.5px] leading-relaxed">{task.description}</p>}

      {task.volume?.steps ? (
        <Section label={t("steps")} note={`${task.volume.steps_done ?? 0}/${task.volume.steps}`} />
      ) : null}
      {task.steps?.map((step) => (
        <div key={step.id} className="flex items-start gap-2.5 py-1 text-[13.5px]">
          <span
            className={`mt-0.5 h-[15px] w-[15px] shrink-0 rounded border-[1.5px] ${
              step.done ? "border-accent bg-accent" : "border-faint"
            }`}
          />
          <span className={step.done ? "text-faint line-through" : ""}>{step.text}</span>
        </div>
      ))}

      {task.volume?.journal ? (
        <Section label={t("journal")} note={String(task.volume.journal)} />
      ) : null}
      {task.log?.map((entry) => (
        <div key={entry.id} className="border-t border-hair py-2.5 first:border-t-0">
          <time className="mb-1 block text-[11.5px] text-faint">{whenLabel({
            at: entry.at,
            tz: entry.tz ?? "",
            floating: false,
            has_time: true,
          })}</time>
          <p className="text-[13.5px] leading-relaxed">{entry.body}</p>
        </div>
      ))}
    </>
  );

  if (expanded) {
    return (
      <main className="flex flex-col overflow-hidden">
        <div data-tauri-drag-region className="h-9 shrink-0" />
        <div className="flex px-6 text-[13px]">
          <button onClick={onCollapse} className="text-accent">
            ‹ {t("collapse")}
          </button>
        </div>
        <div className="scroller mx-auto w-full max-w-[720px] flex-1 px-6 pt-4 pb-12">{body}</div>
      </main>
    );
  }

  return (
    <aside className="flex flex-col overflow-hidden border-l border-hair bg-panel">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="flex gap-4 px-5 text-[13px] text-faint">
        <button onClick={onExpand} title={t("expand")} className="hover:text-accent">
          ⤢
        </button>
      </div>
      <div className="scroller flex-1 px-5 pt-2.5 pb-7">{body}</div>
    </aside>
  );
}

function Chip({ children, tone }: { children: React.ReactNode; tone?: "accent" | "urgent" }) {
  const paint =
    tone === "accent"
      ? "bg-accent-soft text-accent"
      : tone === "urgent"
        ? "bg-urgent/12 text-urgent"
        : "bg-hover text-soft";

  return (
    <span className={`inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs ${paint}`}>
      {children}
    </span>
  );
}

function Section({ label, note }: { label: string; note: string }) {
  return (
    <div className="mt-5 mb-2 flex items-center gap-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
      <span>{label}</span>
      <span className="h-px flex-1 bg-hair" />
      <span>{note}</span>
    </div>
  );
}
