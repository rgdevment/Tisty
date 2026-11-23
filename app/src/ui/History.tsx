import type { Task } from "../core";
import { useSigning } from "../crediting";
import { wroteAt } from "../format";
import { fill, t } from "../locales";
import Signature from "./Signature";
import Written from "./Written";

interface Props {
  task: Task;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
}

const noted = (task: Task) => (task.log ?? []).filter((entry) => entry.body.trim());

/// The page no longer copies the journal, so the frame beside it points down to where it is.
export function JournalLink({ task }: { task: Task }) {
  const count = noted(task).length;
  if (count === 0) return null;
  const reach = (event: React.MouseEvent<HTMLButtonElement>) => {
    const sought = `details[data-journal="${task.id}"]`;
    // The nearest panel holding this task's history, so a second panel open on it is left alone.
    let around = event.currentTarget.parentElement;
    while (around && !around.querySelector(sought)) around = around.parentElement;
    const held = around?.querySelector<HTMLDetailsElement>(sought);
    if (!held) return;
    held.open = true;
    held.scrollIntoView?.({ block: "start", behavior: "smooth" });
  };
  return (
    <div className="mb-1 flex justify-end">
      <button
        type="button"
        onClick={reach}
        className="flex items-center gap-1 rounded-md px-1 text-[11.5px] text-faint hover:text-accent"
      >
        <span aria-hidden="true">↓</span>
        {count === 1 ? t("journalOne") : fill("journalMany", String(count))}
      </button>
    </div>
  );
}

export default function History({ task, onError, onDoc }: Props) {
  const signing = useSigning("inside");
  const kept = noted(task);
  if (kept.length === 0) return null;
  const replaced = new Set(kept.flatMap((entry) => (entry.replaces ? [entry.replaces] : [])));
  return (
    <details open data-journal={task.id} className="mt-6 border-t border-hair pt-3">
      <summary className="cursor-pointer text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase select-none hover:text-soft">
        {t("history")} · {kept.length}
      </summary>
      <ul className="mt-3 flex flex-col gap-3">
        {kept.map((entry) => (
          <li key={entry.id} className={replaced.has(entry.id) ? "opacity-60" : undefined}>
            <span className="flex items-baseline gap-1.5 text-[11.5px] tabular-nums text-faint">
              {wroteAt(entry.at, entry.tz)}
              <Signature by={entry.by} via={entry.via} signing={signing} />
              {replaced.has(entry.id) && <span>· {t("historyReplaced")}</span>}
            </span>
            <Written
              label={t("history")}
              onError={onError}
              onDoc={onDoc}
              text={entry.body}
              steps={task.steps?.map((one) => one.text)}
              className="prose text-[13px] leading-relaxed"
            />
          </li>
        ))}
      </ul>
    </details>
  );
}
