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

  useEffect(() => {
    let live = true;
    taskDocOpen(task.id)
      .then(real)
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
  // A task's document is made by the first words written on it, never by looking at the task.
  const make = doc.file || doc.held ? undefined : (body: string) => taskDocMake(task.id, body);
  return (
    <Written
      key={`${doc.file}:${seen}`}
      id={task.id}
      doc={doc}
      make={make}
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
  id,
  doc,
  make,
  label,
  onSettle,
  onError,
  onDoc,
  onMoved,
  shaping,
}: {
  id: string;
  doc: TaskDoc;
  make?: (body: string) => Promise<string>;
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
  const making = useRef(make);
  making.current = make;
  const unsettled = useRef<string | null>(null);
  const line = useRef<Promise<void>>(Promise.resolve());
  const file = useRef(doc.file);
  const made = useRef<Promise<string> | null>(null);
  // Once the disk has moved on, what was typed over the old words is not written over the new.
  const stale = useRef(false);
  const away = () => {
    stale.current = true;
    moved.current();
  };

  // The first words written make the file; whatever comes after waits for it to be there.
  const ensure = useCallback((body: string): Promise<string> => {
    if (file.current) return Promise.resolve(file.current);
    const asked = making.current;
    if (!asked) return Promise.reject(new Error("unwritable"));
    made.current ??= asked(body).then((one) => {
      file.current = one;
      return one;
    });
    return made.current;
  }, []);

  // Settles wait their turn, so two of them never weigh the same old text and add a step twice.
  const settled = (before: string, now: string) => {
    if (!onSettle) return;
    line.current = line.current.then(async () => {
      const from = unsettled.current ?? before;
      const problem = await ensure(now)
        .then(() => onSettle(from, now))
        .catch((e: unknown) => {
          if ((e as { code?: string } | null)?.code === "documentMoved") return "documentMoved";
          failed.current?.(e);
          return "failed";
        });
      if (problem === null) {
        if (unsettled.current === from) unsettled.current = null;
        return;
      }
      // A settle that failed left its lines unlearnt, so the next one still starts from its text.
      unsettled.current ??= from;
      if (problem === "documentMoved") away();
    });
  };

  const flush = useCallback(() => {
    if (timer.current) clearTimeout(timer.current);
    timer.current = null;
    const next = waiting.current;
    waiting.current = null;
    if (next === null || stale.current) return;
    queued(`words:${id}`, async () => {
      const first = !file.current && !made.current;
      const into = await ensure(next);
      if (!first) await docWrite(into, next);
    }).catch((e) => {
      failed.current?.(e);
      // What is on disk now is newer than what was typed, so the page reads it again.
      if ((e as { code?: string } | null)?.code !== "documentMoved") return;
      stale.current = true;
      moved.current();
    });
  }, [id, ensure]);

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
        file={doc.file || `unwritten:${id}`}
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
