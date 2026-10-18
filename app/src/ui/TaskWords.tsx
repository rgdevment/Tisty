import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";
import {
  addPart,
  docWrite,
  type Filed,
  stepDates,
  type Task,
  type TaskDoc,
  taskDocMake,
  taskDocOpen,
} from "../core";
import { WROTE } from "../detailing";
import { stamped } from "../format";
import { fill } from "../locales";
import { holds, queued } from "../saving";
import { useKeptHeight } from "./heights";
import { usePartCards } from "./partcards";
import Skeleton from "./Skeleton";
import type { Asking } from "./slashing";
import { useTaskAsks } from "./taskAsking";

const Sheet = lazy(() => import("./Sheet"));

const SETTLES = 700;

interface Props {
  task: Task;
  onSettle?: (
    before: string,
    now: string,
    renamed?: readonly [string, string][],
  ) => Promise<string | null>;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  shaping?: boolean;
  onHeld?: (held: boolean) => void;
  papers?: readonly Filed[];
  onTask?: (id: string) => void;
}

// The core refuses the rest the same way: a part holds no parts, and a routine holds none.
const holdsParts = (task: Task): boolean =>
  task.status === "open" && !task.hidden && !task.part_of && !task.repeat && !task.after;

// A command the window does not know answers with whatever it holds, so only a real document counts.
const real = (said: unknown): TaskDoc | null => {
  const one = said as Partial<TaskDoc> | null;
  return typeof one?.file === "string" && typeof one.body === "string" ? (one as TaskDoc) : null;
};

export default function TaskWords({
  task,
  onSettle,
  onError,
  onDoc,
  shaping,
  onHeld,
  papers,
  onTask,
}: Props) {
  const [doc, setDoc] = useState<TaskDoc | null>(null);
  const [seen, setSeen] = useState(0);
  const [turn, setTurn] = useState(0);
  const failed = useRef(onError);
  failed.current = onError;
  const holding = useRef(onHeld);
  holding.current = onHeld;
  const open = task.status === "open";
  const kept = useKeptHeight<HTMLDivElement>(`task:${task.id}`);
  const { asking, picker } = useTaskAsks(open ? papers : undefined, onError);
  const [dates, setDates] = useState<Record<string, string>>({});
  const steps = (task.steps ?? []).map((one) => `${one.text}:${one.done}`).join("|");
  useEffect(() => {
    let live = true;
    if (!steps) return setDates({});
    stepDates(task.id)
      .then((all) => {
        if (!live) return;
        const said: Record<string, string> = {};
        for (const one of all) {
          const parts = [one.born && fill("stepBorn", stamped(one.born))];
          if (one.done) parts.push(fill("stepTicked", stamped(one.done)));
          const line = parts.filter(Boolean).join(" · ");
          if (line) said[one.text] = line;
        }
        setDates(said);
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [task.id, steps]);

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

  // While the page is read, its room stays as it last was, so what is under it never jumps.
  if (!doc)
    return (
      <div ref={kept.ref} className="task-words relative" style={{ minHeight: kept.height ?? 180 }}>
        <Skeleton inset="46px 32px" />
      </div>
    );
  // A task's document is made by the first words written on it, never by looking at the task.
  const make = doc.file || doc.held ? undefined : (body: string) => taskDocMake(task.id, body);
  return (
    <div ref={kept.ref} className="task-words">
      <Written
        key={`${doc.file}:${seen}`}
        task={task}
        doc={doc}
        make={make}
        label={task.title}
        onSettle={doc.held ? undefined : onSettle}
        onError={onError}
        onDoc={onDoc}
        onMoved={() => setTurn((one) => one + 1)}
        shaping={shaping}
        room={kept.height}
        asking={doc.held ? undefined : asking}
        dates={dates}
        onTask={onTask}
        onPart={
          doc.held || !holdsParts(task)
            ? undefined
            : (title) => addPart(task.id, title).then((one) => one.id)
        }
      />
      {picker}
    </div>
  );
}

function Written({
  task,
  doc,
  make,
  label,
  onSettle,
  onError,
  onDoc,
  onMoved,
  shaping,
  room,
  asking,
  dates,
  onTask,
  onPart,
}: {
  task: Task;
  doc: TaskDoc;
  make?: (body: string) => Promise<string>;
  label: string;
  onSettle?: (
    before: string,
    now: string,
    renamed?: readonly [string, string][],
  ) => Promise<string | null>;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  onMoved: () => void;
  shaping?: boolean;
  room?: number;
  asking?: Asking;
  dates?: Readonly<Record<string, string>>;
  onTask?: (id: string) => void;
  onPart?: (title: string) => Promise<string | null>;
}) {
  const [text, setText] = useState(doc.body);
  const cards = usePartCards(task, text);
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
    made.current ??= asked(body).then(
      (one) => {
        file.current = one;
        return one;
      },
      (e: unknown) => {
        made.current = null;
        throw e;
      },
    );
    return made.current;
  }, []);

  // Settles wait their turn, so two of them never weigh the same old text and add a step twice.
  const settled = (before: string, now: string, renamed?: readonly [string, string][]) => {
    if (!onSettle) return;
    line.current = line.current.then(async () => {
      const from = unsettled.current ?? before;
      const problem = await ensure(now)
        .then(() => onSettle(from, now, renamed))
        .catch((e: unknown) => {
          if ((e as { code?: string } | null)?.code === "documentMoved") return "documentMoved";
          failed.current?.(e);
          return "failed";
        });
      if (problem === null || problem === WROTE) {
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
    queued(`words:${task.id}`, async () => {
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
  }, [task.id, ensure]);

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
    <Suspense fallback={<div style={{ minHeight: room }} />}>
      <Sheet
        file={doc.file || `unwritten:${task.id}`}
        value={text}
        label={label}
        reading={doc.held}
        inline
        shaping={shaping}
        onWrite={wrote}
        onSettle={onSettle ? settled : undefined}
        onFail={onError}
        onDoc={onDoc}
        asking={asking}
        stepDates={dates}
        part={cards.part}
        again={`${cards.said}|${onPart ? "parts" : ""}`}
        onTask={onTask}
        onPart={onPart}
      />
    </Suspense>
  );
}
