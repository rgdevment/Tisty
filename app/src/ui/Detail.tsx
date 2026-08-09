import { useEffect, useState } from "react";
import type { Change, List, Task } from "../core";
import { whenLabel } from "../format";
import { t } from "../locales";
import Fields from "./Fields";

interface Props {
  task: Task;
  lists: List[];
  known: string[];
  expanded: boolean;
  onExpand: () => void;
  onCollapse: () => void;
  onPatch: (change: Change) => void;
}

export default function Detail({
  task,
  lists,
  known,
  expanded,
  onExpand,
  onCollapse,
  onPatch,
}: Props) {
  const body = (
    <>
      <Title task={task} big={expanded} onRename={(title) => onPatch({ title })} />
      <Fields task={task} lists={lists} known={known} onPatch={onPatch} />

      <Section label={t("description")} />
      <Wrote task={task} onWrite={(description) => onPatch({ description })} />

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

function Wrote({ task, onWrite }: { task: Task; onWrite: (body: string) => void }) {
  const [text, setText] = useState(task.description ?? "");
  useEffect(() => setText(task.description ?? ""), [task.id, task.description]);

  return (
    <textarea
      rows={2}
      value={text}
      placeholder={t("describeIt")}
      aria-label={t("description")}
      onChange={(e) => setText(e.target.value)}
      onBlur={() => {
        if (text.trim() !== (task.description ?? "").trim()) onWrite(text);
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          setText(task.description ?? "");
          e.currentTarget.blur();
        }
      }}
      className="field-sizing-content w-full resize-none rounded-md bg-transparent px-1.5 py-1 text-[13.5px] leading-relaxed outline-none placeholder:text-faint hover:bg-hover focus:bg-hover"
    />
  );
}

function Title({
  task,
  big,
  onRename,
}: {
  task: Task;
  big: boolean;
  onRename: (title: string) => void;
}) {
  const [text, setText] = useState(task.title);
  const size = big ? "text-[22px]" : "text-[17px]";

  useEffect(() => setText(task.title), [task.id, task.title]);

  const settle = () => {
    const kept = text.trim();
    if (kept && kept !== task.title) onRename(kept);
    else setText(task.title);
  };

  return (
    <textarea
      rows={1}
      value={text}
      aria-label={t("fieldTitle")}
      onChange={(e) => setText(e.target.value)}
      onBlur={settle}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          e.currentTarget.blur();
        }
        if (e.key === "Escape") {
          setText(task.title);
          e.currentTarget.blur();
        }
      }}
      className={`mb-3 field-sizing-content w-full resize-none rounded-md bg-transparent leading-snug font-semibold -tracking-[0.01em] outline-none hover:bg-hover focus:bg-hover ${size}`}
    />
  );
}

function Section({ label, note }: { label: string; note?: string }) {
  return (
    <div className="mt-5 mb-1.5 flex items-center gap-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
      <span>{label}</span>
      <span className="h-px flex-1 bg-hair" />
      {note && <span>{note}</span>}
    </div>
  );
}
