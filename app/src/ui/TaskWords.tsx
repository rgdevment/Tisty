import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import { docWrite, type Task, type TaskDoc, taskDocMake, taskDocOpen } from "../core";
import { t } from "../locales";
import { holds, queued } from "../saving";

const Sheet = lazy(() => import("./Sheet"));

const SETTLES = 700;

interface Props {
  task: Task;
  onSettle?: (before: string, now: string) => Promise<string | null>;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  shaping?: boolean;
  onHeld?: (held: boolean) => void;
}

const templateOf = (task: Task): string =>
  t(task.repeat || task.after ? "templateRoutine" : "templateTask");

// A command the window does not know answers with whatever it holds, so only a real document counts.
const real = (said: unknown): TaskDoc | null => {
  const one = said as Partial<TaskDoc> | null;
  return typeof one?.file === "string" && typeof one.body === "string" ? (one as TaskDoc) : null;
};

export default function TaskWords({ task, onSettle, onError, onDoc, shaping, onHeld }: Props) {
  const [doc, setDoc] = useState<TaskDoc | null>(null);
  const [seen, setSeen] = useState(0);
  const [turn, setTurn] = useState(0);
  const failed = useRef(onError);
  failed.current = onError;
  const holding = useRef(onHeld);
  holding.current = onHeld;
  const open = task.status === "open";
  const template = useRef("");
  template.current = templateOf(task);

  useEffect(() => {
    let live = true;
    const opened = () => taskDocOpen(task.id).then(real);
    opened()
      .then((found) =>
        found || !open ? found : taskDocMake(task.id, template.current).then(opened),
      )
      .then((found) => {
        if (!live) return;
        if (found) holding.current?.(found.held);
        setDoc(found);
        setSeen((one) => one + 1);
      })
      .catch((e) => {
        if (live) failed.current?.(e);
      });
    return () => {
      live = false;
    };
  }, [task.id, open, turn]);

  if (!doc) return null;
  return (
    <Written
      key={`${doc.file}:${seen}`}
      doc={doc}
      label={task.title}
      onSettle={doc.held ? undefined : onSettle}
      onError={onError}
      onDoc={onDoc}
      onMoved={() => setTurn((one) => one + 1)}
      shaping={shaping}
    />
  );
}

function Written({
  doc,
  label,
  onSettle,
  onError,
  onDoc,
  onMoved,
  shaping,
}: {
  doc: TaskDoc;
  label: string;
  onSettle?: (before: string, now: string) => Promise<string | null>;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  onMoved: () => void;
  shaping?: boolean;
}) {
  const [text, setText] = useState(doc.body);
  const waiting = useRef<string | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const failed = useRef(onError);
  failed.current = onError;
  const moved = useRef(onMoved);
  moved.current = onMoved;
  const unsettled = useRef<string | null>(null);
  const line = useRef<Promise<void>>(Promise.resolve());
  // Once the disk has moved on, what was typed over the old words is not written over the new.
  const stale = useRef(false);
  const away = () => {
    stale.current = true;
    moved.current();
  };

  // Settles wait their turn, so two of them never weigh the same old text and add a step twice.
  const settled = (before: string, now: string) => {
    if (!onSettle) return;
    line.current = line.current
      .then(async () => {
        const from = unsettled.current ?? before;
        const problem = await onSettle(from, now);
        if (problem === null) {
          if (unsettled.current === from) unsettled.current = null;
          return;
        }
        // A settle that failed left its lines unlearnt, so the next one still starts from its text.
        unsettled.current ??= from;
        if (problem === "documentMoved") away();
      })
      .catch(() => {});
  };

  const flush = useCallback(() => {
    if (timer.current) clearTimeout(timer.current);
    timer.current = null;
    const next = waiting.current;
    waiting.current = null;
    if (next === null || stale.current) return;
    queued(doc.file, () => docWrite(doc.file, next)).catch((e) => {
      failed.current?.(e);
      // What is on disk now is newer than what was typed, so the page reads it again.
      if ((e as { code?: string } | null)?.code !== "documentMoved") return;
      stale.current = true;
      moved.current();
    });
  }, [doc.file]);

  useEffect(() => holds(flush), [flush]);
  useEffect(() => {
    window.addEventListener("blur", flush);
    return () => {
      window.removeEventListener("blur", flush);
      flush();
    };
  }, [flush]);

  const wrote = (next: string) => {
    setText(next);
    waiting.current = next;
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(flush, SETTLES);
  };

  return (
    <Suspense fallback={<p className="text-[12.5px] text-faint">{t("opening")}</p>}>
      <Sheet
        file={doc.file}
        value={text}
        label={label}
        reading={doc.held}
        inline
        shaping={shaping}
        onWrite={wrote}
        onSettle={onSettle ? settled : undefined}
        onFail={onError}
        onDoc={onDoc}
      />
    </Suspense>
  );
}
