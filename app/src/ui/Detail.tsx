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
  expanded: boolean;
  from?: string;
  onExpand: () => void;
  onCollapse: () => void;
  onPatch: (change: Change) => void;
  onStep: (text: string, step?: string) => void;
  onMark: (step: string, done: boolean) => void;
  onDropStep: (step: string) => void;
  onLog: (body: string, entry?: string) => void;
  onComplete: () => void;
  onDiscard: () => void;
  onReopen: () => void;
  onErase: () => void;
  onClose: () => void;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
}

export default function Detail({
  task,
  lists,
  known,
  expanded,
  from,
  onExpand,
  onCollapse,
  onPatch,
  onStep,
  onMark,
  onDropStep,
  onLog,
  onComplete,
  onDiscard,
  onReopen,
  onErase,
  onClose,
  onError,
  onDoc,
}: Props) {
  const opened = useRef<HTMLElement>(null);
  useEffect(() => {
    opened.current?.focus({ preventScroll: true });
  }, [task.id]);

  const leave = (event: React.KeyboardEvent) => {
    if (event.key !== "Escape") return;
    const at = event.target as HTMLElement;
    if (at.isContentEditable || at.closest("input, textarea, select")) return;
    event.preventDefault();
    onClose();
  };

  const body = (
    <>
      <Title task={task} big={expanded} onRename={(title) => onPatch({ title })} />
      <Fields task={task} lists={lists} known={known} onPatch={onPatch} />

      <Section label={t("description")} />
      <Prose
        value={task.description ?? ""}
        hint={t("describeIt")}
        label={t("description")}
        beside={expanded}
        catches
        onError={onError}
        onDoc={onDoc}
        onWhole={expanded ? undefined : onExpand}
        onWrite={(description) => onPatch({ description })}
      />

      <Section
        label={t("steps")}
        note={
          task.volume?.steps ? `${task.volume.steps_done ?? 0}/${task.volume.steps}` : undefined
        }
      />
      <Steps steps={task.steps ?? []} onWrite={onStep} onMark={onMark} onDrop={onDropStep} />

      <Section
        label={t("journal")}
        note={task.volume?.journal ? String(task.volume.journal) : undefined}
      />
      <Journal
        entries={task.log ?? []}
        steps={task.steps?.map((one) => one.text)}
        onError={onError}
        onDoc={onDoc}
        onWhole={expanded ? undefined : onExpand}
        onWrite={onLog}
      />
    </>
  );

  if (expanded) {
    return (
      <main
        ref={opened}
        tabIndex={-1}
        onKeyDown={leave}
        className="flex flex-col overflow-hidden outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
      >
        <div data-tauri-drag-region className="h-9 shrink-0" />
        <div className="flex items-center gap-1 px-6 text-[13px] text-faint">
          <button
            type="button"
            onClick={onCollapse}
            aria-label={from ? `${t("collapse")} — ${from}` : t("collapse")}
            className="-ml-2 rounded-md px-2 py-1 text-accent hover:bg-hover"
          >
            <span aria-hidden="true">‹</span> {from || t("collapse")}
          </button>
        </div>
        <div className="scroller mx-auto w-full max-w-[720px] flex-1 px-6 pt-4 pb-12">{body}</div>
        <Settled
          task={task}
          wide
          onComplete={onComplete}
          onDiscard={onDiscard}
          onReopen={onReopen}
          onErase={onErase}
        />
      </main>
    );
  }

  return (
    <aside
      ref={opened as React.RefObject<HTMLElement>}
      tabIndex={-1}
      onKeyDown={leave}
      className="flex flex-col overflow-hidden border-l border-hair bg-panel outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
    >
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="flex items-center gap-1 px-5 text-[13px] text-faint">
        <button
          type="button"
          onClick={onClose}
          title={t("closePanel")}
          aria-label={t("closePanel")}
          aria-keyshortcuts="Escape"
          className="-ml-1 flex h-6 w-6 items-center justify-center rounded-md hover:bg-hover hover:text-ink"
        >
          <span aria-hidden="true">✕</span>
        </button>
        <button
          type="button"
          onClick={onExpand}
          title={t("expand")}
          aria-label={t("expand")}
          className="flex h-6 w-6 items-center justify-center rounded-md hover:bg-hover hover:text-accent"
        >
          <span aria-hidden="true">⤢</span>
        </button>
      </div>
      <div className="scroller flex-1 px-5 pt-2.5 pb-7">{body}</div>
      <Settled
        task={task}
        onComplete={onComplete}
        onDiscard={onDiscard}
        onReopen={onReopen}
        onErase={onErase}
      />
    </aside>
  );
}

function Settled({
  task,
  wide,
  onComplete,
  onDiscard,
  onReopen,
  onErase,
}: {
  task: Task;
  wide?: boolean;
  onComplete: () => void;
  onDiscard: () => void;
  onReopen: () => void;
  onErase: () => void;
}) {
  const folded = task.hidden || task.status === "dropped";
  const seat = "flex items-center gap-1 rounded-md px-2.5 py-1 hover:bg-hover";

  return (
    <footer
      aria-label={t("taskDoings")}
      className="shrink-0 border-hair border-t bg-panel/70 px-3 py-1.5 text-[12.5px] text-soft backdrop-blur"
    >
      <div
        className={
          wide ? "mx-auto flex w-full max-w-[720px] items-center gap-1" : "flex items-center gap-1"
        }
      >
        {task.status === "open" ? (
          <>
            <button
              type="button"
              onClick={onComplete}
              className={`${seat} font-medium text-accent`}
            >
              <span aria-hidden="true">✓</span> {t("markDone")}
            </button>
            <button
              type="button"
              onClick={onDiscard}
              title={task.repeat ? t("endRepeatWhy") : undefined}
              className={`${seat} hover:text-ink`}
            >
              <span aria-hidden="true">⊘</span> {task.repeat ? t("endRepeat") : t("discardIt")}
            </button>
          </>
        ) : (
          <>
            <button type="button" onClick={onReopen} className={`${seat} hover:text-ink`}>
              <span aria-hidden="true">↺</span> {t("reopenIt")}
            </button>
            {folded && (
              <button
                type="button"
                onClick={onErase}
                className={`${seat} ml-auto text-faint hover:text-urgent`}
              >
                <span aria-hidden="true">✕</span> {t("eraseIt")}
              </button>
            )}
          </>
        )}
      </div>
    </footer>
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
