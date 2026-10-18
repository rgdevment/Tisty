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
}

// A command the window does not know answers with whatever it holds, so only a real document counts.
const templateOf = (task: Task): string =>
  t(task.repeat || task.after ? "templateRoutine" : "templateTask");

const real = (said: unknown): TaskDoc | null => {
  const one = said as Partial<TaskDoc> | null;
  return typeof one?.file === "string" && typeof one.body === "string" ? (one as TaskDoc) : null;
};

export default function TaskWords({ task, onSettle, onError, onDoc, shaping }: Props) {
  const [doc, setDoc] = useState<TaskDoc | null>(null);
  const [seen, setSeen] = useState(0);
  const [turn, setTurn] = useState(0);
  const failed = useRef(onError);
  failed.current = onError;
  const open = task.status === "open";
  const template = templateOf(task);

  useEffect(() => {
    let live = true;
    const opened = () => taskDocOpen(task.id).then(real);
    opened()
      .then((found) => (found || !open ? found : taskDocMake(task.id, template).then(opened)))
      .then((found) => {
        if (!live) return;
        setDoc(found);
        setSeen((one) => one + 1);
      })
      .catch((e) => {
        if (live) failed.current?.(e);
      });
    return () => {
      live = false;
    };
  }, [task.id, open, template, turn]);

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

  // A settle that failed left its lines unlearnt, so the next one still starts from its text.
  const settled = (before: string, now: string) => {
    if (!onSettle) return;
    const from = unsettled.current ?? before;
    void onSettle(from, now).then((problem) => {
      if (problem === null) {
        if (unsettled.current === from) unsettled.current = null;
        return;
      }
      unsettled.current ??= from;
      if (problem === "documentMoved") moved.current();
    });
  };

  const flush = useCallback(() => {
    if (timer.current) clearTimeout(timer.current);
    timer.current = null;
    const next = waiting.current;
    if (next === null) return;
    waiting.current = null;
    queued(doc.file, () => docWrite(doc.file, next)).catch((e) => {
      failed.current?.(e);
      // What is on disk now is newer than what was typed, so the page reads it again.
      if ((e as { code?: string } | null)?.code === "documentMoved") moved.current();
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
