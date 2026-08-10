import { useEffect, useRef, useState } from "react";
import type { Change, List, Task } from "../core";
import { t } from "../locales";
import Fields from "./Fields";
import Journal from "./Journal";
import Prose from "./Prose";
import Steps from "./Steps";

interface Props {
  task: Task;
  lists: List[];
  known: string[];
  /** References already in use; not the same list as the tags. */
  refs: string[];
  expanded: boolean;
  /** Where the reader came from, so «back» names a place and not a layout. */
  from?: string;
  onExpand: () => void;
  onCollapse: () => void;
  onPatch: (change: Change) => void;
  onStep: (text: string, step?: string) => void;
  onMark: (step: string, done: boolean) => void;
  onDropStep: (step: string) => void;
  onLog: (body: string, entry?: string) => void;
  onDiscard: () => void;
  onReopen: () => void;
  onError?: (problem: unknown) => void;
}

export default function Detail({
  task,
  lists,
  known,
  refs,
  expanded,
  from,
  onExpand,
  onCollapse,
  onPatch,
  onStep,
  onMark,
  onDropStep,
  onLog,
  onDiscard,
  onReopen,
  onError,
}: Props) {
  const body = (
    <>
      <Title task={task} big={expanded} onRename={(title) => onPatch({ title })} />
      <Fields task={task} lists={lists} known={known} onPatch={onPatch} />

      <Section label={t("description")} />
      <Prose
        value={task.description ?? ""}
        hint={t("describeIt")}
        label={t("description")}
        known={refs}
        beside={expanded}
        catches
        onError={onError}
        onWhole={expanded ? undefined : onExpand}
        onWrite={(description) => onPatch({ description })}
      />

      <Section
        label={t("steps")}
        note={task.volume?.steps ? `${task.volume.steps_done ?? 0}/${task.volume.steps}` : undefined}
      />
      <Steps steps={task.steps ?? []} onWrite={onStep} onMark={onMark} onDrop={onDropStep} />

      <Section label={t("journal")} note={task.volume?.journal ? String(task.volume.journal) : undefined} />
      <Journal
        entries={task.log ?? []}
        known={refs}
        onError={onError}
        onWhole={expanded ? undefined : onExpand}
        onWrite={onLog}
      />
    </>
  );

  if (expanded) {
    return (
      <main className="flex flex-col overflow-hidden">
        <div data-tauri-drag-region className="h-9 shrink-0" />
        <div className="flex items-center gap-1 px-6 text-[13px] text-faint">
          <button
            onClick={onCollapse}
            className="-ml-2 rounded-md px-2 py-1 text-accent hover:bg-hover"
          >
            ‹ {from || t("collapse")}
          </button>
          <Settled task={task} onDiscard={onDiscard} onReopen={onReopen} />
        </div>
        <div className="scroller mx-auto w-full max-w-[720px] flex-1 px-6 pt-4 pb-12">{body}</div>
      </main>
    );
  }

  return (
    <aside className="flex flex-col overflow-hidden border-l border-hair bg-panel">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="flex items-center gap-1 px-5 text-[13px] text-faint">
        <button
          onClick={onExpand}
          title={t("expand")}
          aria-label={t("expand")}
          className="flex h-6 w-6 items-center justify-center rounded-md hover:bg-hover hover:text-accent"
        >
          ⤢
        </button>
        <Settled task={task} onDiscard={onDiscard} onReopen={onReopen} />
      </div>
      <div className="scroller flex-1 px-5 pt-2.5 pb-7">{body}</div>
    </aside>
  );
}

function Settled({
  task,
  onDiscard,
  onReopen,
}: {
  task: Task;
  onDiscard: () => void;
  onReopen: () => void;
  onError?: (problem: unknown) => void;
}) {
  const shut = task.status !== "open";
  return (
    <button
      onClick={shut ? onReopen : onDiscard}
      className="ml-auto rounded-md px-2 py-1 hover:bg-hover hover:text-ink"
    >
      {shut ? `↺ ${t("reopenIt")}` : `⊘ ${t("discardIt")}`}
    </button>
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
  const dropped = useRef(false);
  const size = big ? "text-[22px]" : "text-[17px]";

  useEffect(() => setText(task.title), [task.id, task.title]);

  const settle = () => {
    if (dropped.current) {
      dropped.current = false;
      setText(task.title);
      return;
    }
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
          dropped.current = true;
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
