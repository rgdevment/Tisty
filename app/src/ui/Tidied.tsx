import { type CSSProperties, useEffect, useState } from "react";
import { docUnflag, type Filed, type Task, taskOf } from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import {
  alike,
  compactedBy,
  dropWord,
  listOf,
  looksLikeTask,
  plainWord,
  pointedWord,
} from "../tidying";
import { clientNamed } from "../who";

const BAR = "mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]";
const MILD = "rounded-[10px] border border-line px-2 py-0.5 text-[11.5px] hover:bg-hover";
const RISKY = `${MILD} text-urgent`;

type Target = { doc: Filed } | { task: Task } | "gone" | null;

const useTarget = (instead: string | null | undefined, known: Filed[]): Target => {
  const doc = instead
    ? known.find((one) => (one.file === instead || one.id === instead) && !one.gone)
    : undefined;
  const [task, setTask] = useState<Task | "gone" | null>(null);
  const asks = !doc && instead && looksLikeTask(instead) ? instead : null;
  useEffect(() => {
    setTask(null);
    if (!asks) return;
    let live = true;
    taskOf(asks).then(
      (one) => live && setTask(one),
      () => live && setTask("gone"),
    );
    return () => {
      live = false;
    };
  }, [asks]);
  if (doc) return { doc };
  if (!asks) return "gone";
  return task === null ? null : task === "gone" ? "gone" : { task };
};

interface Reached {
  onDoc?: (file: string) => void;
  onTask?: (task: Task) => void;
}

function Reason({ own, known, onDoc, onTask }: { own: Filed; known: Filed[] } & Reached) {
  const mark = own.flagged;
  const target = useTarget(mark?.instead, known);
  const pointed = pointedWord(mark?.reason);
  if (!pointed) {
    const plain = plainWord(mark?.reason);
    return plain ? t(plain) : null;
  }
  const [before, after = ""] = t(pointed).split("{name}");
  const there =
    target === null ? (
      "…"
    ) : target === "gone" ? (
      t("flagNoLonger")
    ) : (
      <button
        type="button"
        onClick={() => ("doc" in target ? onDoc?.(target.doc.file) : onTask?.(target.task))}
        className="underline underline-offset-2 hover:text-ink"
      >
        {"doc" in target ? target.doc.title || t("untitledDoc") : target.task.title}
      </button>
    );
  return (
    <>
      {before}
      {there}
      {after}
    </>
  );
}

interface MarkProps extends Reached {
  own: Filed;
  known: Filed[];
  style: CSSProperties;
  onKept: () => void;
  onError: (problem: unknown) => void;
  onAway: () => void;
  onDrop?: (doc: Filed) => void;
  onDropAll?: (docs: Filed[]) => void;
}

export function MarkBar({
  own,
  known,
  style,
  onKept,
  onError,
  onAway,
  onDrop,
  onDropAll,
  onDoc,
  onTask,
}: MarkProps) {
  const mark = own.flagged;
  if (!mark) return null;
  const named = clientNamed(mark.via);
  const who = named ? fill("docFlaggedBy", named) : t("docFlagged");
  const reasoned = Boolean(pointedWord(mark.reason) || plainWord(mark.reason));
  const set = alike(known, own);
  const many = String(set.length);
  const unmarkAll = async () => {
    if (!(await sure(fill("flagUnmarkAllSure", many, listOf(set))))) return;
    Promise.all(set.map((one) => docUnflag(one.id)))
      .then(onKept)
      .catch((e) => onError(saidPlainly(e)));
  };
  return (
    <div style={style} className={BAR}>
      <span className="font-medium text-hue-teal">
        <span aria-hidden="true">◆ </span>
        {reasoned ? <Reason own={own} known={known} onDoc={onDoc} onTask={onTask} /> : who}
      </span>
      {reasoned && <span className="text-soft">{who}</span>}
      <span className="text-faint">{stamped(mark.at)}</span>
      <button type="button" onClick={onAway} className={MILD}>
        {t("putAway")}
      </button>
      {onDrop && !own.locked && (
        <button type="button" onClick={() => onDrop(own)} className={RISKY}>
          {t(dropWord(mark.reason))}
        </button>
      )}
      <button
        type="button"
        onClick={() =>
          docUnflag(own.id)
            .then(onKept)
            .catch((e) => onError(saidPlainly(e)))
        }
        className={MILD}
      >
        {t("unflagIt")}
      </button>
      <span className="w-full text-soft">{mark.said}</span>
      {set.length > 1 && (
        <span className="flex w-full flex-wrap gap-x-3 gap-y-1">
          {onDropAll && !set.some((one) => one.locked) && (
            <button type="button" onClick={() => onDropAll(set)} className={RISKY}>
              {fill("flagDropAll", many)}
            </button>
          )}
          <button type="button" onClick={() => void unmarkAll()} className={MILD}>
            {fill("flagUnmarkAll", many)}
          </button>
        </span>
      )}
    </div>
  );
}

interface CompactProps {
  own: Filed;
  style: CSSProperties;
  backable: boolean;
  onUndo: () => void;
  onKeep: () => void;
}

export function CompactBar({ own, style, backable, onUndo, onKeep }: CompactProps) {
  if (!own.compacted) return null;
  return (
    <div style={style} className={BAR}>
      <span className="font-medium text-hue-teal">
        <span aria-hidden="true">◇ </span>
        {compactedBy(own)}
      </span>
      {own.compacted.last && backable && (
        <button type="button" onClick={onUndo} className={MILD}>
          {t("compactUndo")}
        </button>
      )}
      <button type="button" onClick={onKeep} className={MILD}>
        {t("compactKeep")}
      </button>
    </div>
  );
}
