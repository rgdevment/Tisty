import { lazy, type ReactNode, Suspense, useCallback, useEffect, useRef, useState } from "react";
import { docWrite, type Task, type TaskDoc, taskDocMake, taskDocOpen } from "../core";
import { t } from "../locales";
import { holds, queued } from "../saving";

const Sheet = lazy(() => import("./Sheet"));

const SETTLES = 700;

interface Props {
  task: Task;
  legacy: ReactNode;
  onSettle?: (before: string, now: string) => void;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
}

// A command the window does not know answers with whatever it holds, so only a real document counts.
const real = (said: unknown): TaskDoc | null => {
  const one = said as Partial<TaskDoc> | null;
  return typeof one?.file === "string" && typeof one.body === "string" ? (one as TaskDoc) : null;
};

export default function TaskWords({ task, legacy, onSettle, onError, onDoc }: Props) {
  const [doc, setDoc] = useState<TaskDoc | null | undefined>(undefined);
  const failed = useRef(onError);
  failed.current = onError;

  const look = useCallback(
    () =>
      taskDocOpen(task.id)
        .then((said) => setDoc(real(said)))
        .catch((e) => {
          setDoc(null);
          failed.current?.(e);
        }),
    [task.id],
  );

  useEffect(() => {
    void look();
  }, [look]);

  if (!doc) {
    return (
      <>
        {legacy}
        {doc === null && onSettle && task.status === "open" && (
          <button
            type="button"
            onClick={() =>
              taskDocMake(task.id)
                .then(look)
                .catch((e) => onError?.(e))
            }
            className="mt-4 rounded-md px-2 py-1 text-[12.5px] text-accent hover:bg-hover"
          >
            {t("writeAsDoc")}
          </button>
        )}
      </>
    );
  }
  return (
    <Written
      doc={doc}
      label={task.title}
      onSettle={doc.held ? undefined : onSettle}
      onError={onError}
      onDoc={onDoc}
    />
  );
}

function Written({
  doc,
  label,
  onSettle,
  onError,
  onDoc,
}: {
  doc: TaskDoc;
  label: string;
  onSettle?: (before: string, now: string) => void;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
}) {
  const [text, setText] = useState(doc.body);
  const waiting = useRef<string | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const failed = useRef(onError);
  failed.current = onError;

  const flush = useCallback(() => {
    if (timer.current) clearTimeout(timer.current);
    timer.current = null;
    const next = waiting.current;
    if (next === null) return;
    waiting.current = null;
    queued(doc.file, () => docWrite(doc.file, next)).catch((e) => failed.current?.(e));
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
        onWrite={wrote}
        onSettle={onSettle}
        onFail={onError}
        onDoc={onDoc}
      />
    </Suspense>
  );
}
