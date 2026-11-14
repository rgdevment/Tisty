import { emit } from "@tauri-apps/api/event";
import { type ReactNode, useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { ideaToDoc, ideaUnflag } from "../brainstorm";
import { capture, type Filed, type IdeaMarked, type Task } from "../core";
import { fill, locale, t } from "../locales";
import "../notes.css";
import { pinMarkup } from "tisty-editor/read";
import {
  added,
  boardOf,
  bornHeading,
  colourOf,
  dropped,
  IDEA_AT_MOST,
  introOf,
  type Note,
  ordered,
  rewritten,
  saidWhen,
  shapeOf,
  shifted,
  without,
} from "../notes";
import { sure } from "../sure";
import { Left } from "./Field";
import FileIdea from "./FileIdea";
import Glyph from "./Glyph";
import { IdeaMarkBar, IdeaMarkStrip, markedBy, markOf } from "./IdeaMark";

interface Props {
  title: string;
  body: string;
  reading: boolean;
  onWrite: (body: string) => void;
  onEditor?: () => void;
  known?: Filed[];
  own?: string;
  onError?: (problem: unknown) => void;
  onDoc?: (file: string) => void;
  onTask?: (task: Task) => void;
  onMarks?: () => void;
}

interface Lifted {
  from: number;
  x: number;
  y: number;
  dx: number;
  dy: number;
  over: number | null;
}

const LIFTS_PAST = 6;
const REACH = 260;

function nearest(x: number, y: number, from: number): number | null {
  let best: { at: number; far: number } | null = null;
  for (const one of document.querySelectorAll<HTMLElement>("[data-shown]")) {
    const at = Number(one.dataset.shown);
    if (at === from) continue;
    const box = one.getBoundingClientRect();
    const far = Math.hypot(box.left + box.width / 2 - x, box.top + box.height / 2 - y);
    if (far <= REACH && (!best || far < best.far)) best = { at, far };
  }
  return best?.at ?? null;
}

export default function Notes({
  title,
  body,
  reading,
  onWrite,
  onEditor,
  known = [],
  own = "",
  onError = () => {},
  onDoc,
  onTask,
  onMarks = () => {},
}: Props) {
  const [sent, send] = useSent();
  const [raw, setRaw] = useState(false);
  const [filing, setFiling] = useState<Note | null>(null);
  const latest = useRef(body);
  latest.current = body;
  const settled = (idea: Note) => () => onWrite(without(latest.current, idea));
  const failed = (problem: unknown) => onError(problem);
  const { notes } = boardOf(body);
  const intro = introOf(body);
  const marks = known.find((one) => one.file === own)?.ideasFlagged ?? [];
  const marked = notes.filter((note) => markOf(marks, note));
  const unmark = (gone: Note[]) =>
    void Promise.all(gone.map((note) => ideaUnflag(own, note.heading, note.text)))
      .then(onMarks)
      .catch(failed);
  const dropMarked = (gone: Note[]) => {
    onWrite(gone.reduce((left, note) => without(left, note), latest.current));
    unmark(gone);
  };
  const dropAllMarked = async () => {
    const listed = marked.map((one) => `«${one.text}»`).join("\n");
    if (marked.length > 1 && !(await sure(fill("ideasDropAllSure", String(marked.length), listed))))
      return;
    dropMarked(marked);
  };
  const stripOf = (note: Note) => {
    const mark = markOf(marks, note);
    return mark ? (
      <IdeaMarkStrip
        mark={mark}
        reading={reading}
        notes={notes}
        known={known}
        onDoc={onDoc}
        onTask={onTask}
        onDrop={() => dropMarked([note])}
        onUnmark={() => unmark([note])}
      />
    ) : null;
  };
  const [fresh, setFresh] = useState<number | null>(null);
  const [lifted, setLifted] = useState<Lifted | null>(null);
  const newestFirst = notes
    .map((note, at) => {
      const nth = notes.slice(0, at).filter((one) => one.heading === note.heading).length;
      const kept = colourOf(note.heading);
      return { note, at, nth, hue: kept >= 0 ? kept : shapeOf(note.heading, nth).hue };
    })
    .reverse();

  const moved = (from: number, to: number) => {
    if (from === to || to < 0 || to >= newestFirst.length) return;
    const shown = shifted(
      newestFirst.map((one) => one.at),
      from,
      to,
    );
    onWrite(ordered(body, [...shown].reverse()));
  };

  const held = useRef(lifted);
  held.current = lifted;
  const settle = useRef(moved);
  settle.current = moved;
  const lifting = lifted !== null;
  const wall = useRef<HTMLUListElement>(null);
  useSlides(wall, newestFirst.map(({ note, nth }) => `${note.heading}#${nth}`).join("|"));
  useEffect(() => {
    if (!lifting) return;
    const drop = () => {
      const was = held.current;
      setLifted(null);
      if (was && was.over !== null && Math.hypot(was.dx, was.dy) > LIFTS_PAST) {
        settle.current(was.from, was.over);
      }
    };
    const cancel = () => setLifted(null);
    const follow = (e: PointerEvent) => {
      const was = held.current;
      if (!was) return;
      // A button let go outside the window sends no pointerup; it comes back with none held.
      if (e.buttons === 0) {
        drop();
        return;
      }
      setLifted({
        ...was,
        dx: e.clientX - was.x,
        dy: e.clientY - was.y,
        over: nearest(e.clientX, e.clientY, was.from),
      });
    };
    globalThis.addEventListener("pointermove", follow);
    globalThis.addEventListener("pointerup", drop);
    globalThis.addEventListener("pointercancel", cancel);
    globalThis.addEventListener("blur", cancel);
    return () => {
      globalThis.removeEventListener("pointermove", follow);
      globalThis.removeEventListener("pointerup", drop);
      globalThis.removeEventListener("pointercancel", cancel);
      globalThis.removeEventListener("blur", cancel);
    };
  }, [lifting]);

  return (
    <section aria-label={title} className="w-full px-8 pt-6 pb-16">
      <div className="flex items-center gap-2">
        <h1 className="min-w-0 flex-1 truncate text-[21px] font-semibold">{title}</h1>
        {!reading && !raw && (
          <button
            type="button"
            onClick={() => {
              setFresh(notes.length);
              onWrite(added(body, bornHeading(body, new Date())));
            }}
            className="rounded-md px-2 py-1 text-[12.5px] text-accent hover:bg-hover"
          >
            {t("notesAdd")}
          </button>
        )}
        <button
          type="button"
          onClick={() => (onEditor ? onEditor() : setRaw((was) => !was))}
          aria-pressed={raw}
          className="rounded-md px-2 py-1 text-[12.5px] text-soft hover:bg-hover"
        >
          {t(raw ? "notesAsNotes" : "notesAsText")}
        </button>
      </div>
      {sent && (
        <p
          role="status"
          className="mt-3 flex items-center gap-3 rounded-md bg-accent-soft px-3 py-1.5 text-[12.5px] text-accent"
        >
          <span className="min-w-0 flex-1 truncate">{sent.text}</span>
          <button type="button" onClick={sent.open} className="shrink-0 font-semibold underline">
            {sent.action}
          </button>
        </p>
      )}
      {!raw && marked.length > 0 && (
        <IdeaMarkBar
          marks={marked.map((note) => markOf(marks, note)).filter((one) => one !== undefined)}
          reading={reading}
          onDropAll={() => void dropAllMarked()}
          onUnmarkAll={() => unmark(marked)}
        />
      )}
      {!raw && intro && <p className="mt-3 text-[12.5px] whitespace-pre-wrap text-soft">{intro}</p>}
      {raw ? (
        <pre className="mt-6 overflow-x-auto rounded-[10px] border border-line bg-bg p-5 font-mono text-[12.5px] whitespace-pre-wrap text-ink">
          {body}
        </pre>
      ) : notes.length === 0 ? (
        <p className="pt-6 text-[12.5px] text-faint">{t("notesNone")}</p>
      ) : (
        <ul ref={wall} className="notes-wall">
          {newestFirst.map(({ note, at, nth, hue }, shown) => (
            <Sticky
              key={`${note.heading}#${nth}#${note.text}`}
              stamp={`${note.heading}#${nth}#${note.text}`}
              note={note}
              nth={nth}
              hue={hue}
              mark={markOf(marks, note)}
              strip={stripOf(note)}
              shown={shown}
              reading={reading}
              focused={fresh === at}
              lift={lifted?.from === shown ? { dx: lifted.dx, dy: lifted.dy } : null}
              target={lifted !== null && lifted.from !== shown && lifted.over === shown}
              onLift={(x, y) => setLifted({ from: shown, x, y, dx: 0, dy: 0, over: null })}
              onMove={(by) => moved(shown, shown + by)}
              onText={(text) => {
                setFresh(null);
                if (!text.trim() && fresh === at) onWrite(dropped(body, at));
                else if (text !== note.text) onWrite(rewritten(body, at, text));
              }}
              onDrop={() => onWrite(dropped(body, at))}
              onFile={() => setFiling(note)}
              onTask={() =>
                void capture(note.text)
                  .then((task) => {
                    settled(note)();
                    void emit("captured");
                    send(fill("ideaMadeTask", task.title), t("ideaSeeTask"), () => onTask?.(task));
                  })
                  .catch(failed)
              }
            />
          ))}
        </ul>
      )}
      {filing && (
        <FileIdea
          idea={filing.text}
          known={known}
          own={own}
          onClose={() => setFiling(null)}
          onPick={(file) => {
            setFiling(null);
            const there = known.find((one) => one.file === file)?.title || t("untitledDoc");
            void ideaToDoc(file, filing.text, filing.heading)
              .then(() => {
                settled(filing)();
                send(fill("ideaFiledIn", there), t("ideaOpenDoc"), () => onDoc?.(file));
              })
              .catch(failed);
          }}
        />
      )}
    </section>
  );
}

const SENT_FOR = 8_000;

interface Sent {
  text: string;
  action: string;
  open: () => void;
}

function useSent(): [Sent | null, (text: string, action: string, open: () => void) => void] {
  const [sent, setSent] = useState<Sent | null>(null);
  const fading = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(fading.current), []);
  const send = (text: string, action: string, open: () => void) => {
    clearTimeout(fading.current);
    setSent({ text, action, open });
    fading.current = setTimeout(() => setSent(null), SENT_FOR);
  };
  return [sent, send];
}

const SLIDES_FOR = 220;

// Measured only when the order changes: measuring on every render forces a layout per pointer move.
function useSlides(wall: React.RefObject<HTMLUListElement | null>, order: string) {
  const were = useRef(new Map<string, DOMRect>());
  useLayoutEffect(() => {
    if (!order) return;
    const still = globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? true;
    const now = new Map<string, DOMRect>();
    for (const one of wall.current?.querySelectorAll<HTMLElement>("[data-key]") ?? []) {
      const box = one.getBoundingClientRect();
      const was = were.current.get(one.dataset.key ?? "");
      const dx = was ? was.left - box.left : 0;
      const dy = was ? was.top - box.top : 0;
      if (!still && (dx || dy)) {
        one.animate?.([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "none" }], {
          duration: SLIDES_FOR,
          easing: "ease-out",
        });
      }
      now.set(one.dataset.key ?? "", box);
    }
    were.current = now;
  }, [order, wall]);
}

interface StickyProps {
  stamp: string;
  note: Note;
  mark?: IdeaMarked;
  strip?: ReactNode;
  nth: number;
  hue: number;
  shown: number;
  reading: boolean;
  focused: boolean;
  lift: { dx: number; dy: number } | null;
  target: boolean;
  onLift: (x: number, y: number) => void;
  onMove: (by: number) => void;
  onText: (text: string) => void;
  onDrop: () => void;
  onFile: () => void;
  onTask: () => void;
}

const EARLIER = new Set(["ArrowUp", "ArrowLeft"]);
const LATER = new Set(["ArrowDown", "ArrowRight"]);

function Sticky({
  stamp,
  note,
  mark,
  strip,
  nth,
  hue,
  shown,
  reading,
  focused,
  lift,
  target,
  onLift,
  onMove,
  onText,
  onDrop,
  onFile,
  onTask,
}: StickyProps) {
  const shape = shapeOf(note.heading, nth);
  const words = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    if (focused) words.current?.focus();
  }, [focused]);
  const lines = note.text.split("\n").length;
  const [used, setUsed] = useState(note.text.length);
  const nudge = shape.nudge * 6;

  return (
    <li
      data-shown={shown}
      data-key={stamp}
      className="sticky-note"
      data-hue={hue}
      data-wide={shape.wide}
      data-hold={shape.hold}
      data-pin={shape.pin}
      data-lifted={lift ? "" : undefined}
      data-target={target ? "" : undefined}
      data-marked={mark ? "" : undefined}
      onPointerDown={(e) => {
        if (reading || e.button !== 0) return;
        if ((e.target as HTMLElement).closest("textarea, button")) return;
        e.preventDefault();
        onLift(e.clientX, e.clientY);
      }}
      onKeyDown={(e) => {
        if (reading || !e.altKey) return;
        const by = EARLIER.has(e.key) ? -1 : LATER.has(e.key) ? 1 : 0;
        if (!by) return;
        e.preventDefault();
        onMove(by);
      }}
      style={
        {
          rotate: lift ? "0deg" : `${shape.tilt}deg`,
          translate: lift ? `${nudge + lift.dx}px ${lift.dy}px` : `${nudge}px 0`,
          marginTop: `${shape.drop * 10}px`,
          marginInline: `${shape.space * 5}px`,
          "--angle": `${shape.angle}deg`,
          "--along": `${shape.along}%`,
        } as React.CSSProperties
      }
    >
      {shape.hold === "pin" ? <Pin /> : <span aria-hidden="true" className="sticky-hold" />}
      {mark && (
        <span className="sticky-corner" title={markedBy(mark.mark)}>
          ◆
        </span>
      )}
      <p className="sticky-when">{saidWhen(note.heading, locale())}</p>
      {reading ? (
        <p className="sticky-words">{note.text}</p>
      ) : (
        <textarea
          ref={words}
          aria-label={t("notesWords")}
          aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown"
          defaultValue={note.text}
          maxLength={IDEA_AT_MOST}
          onChange={(e) => setUsed(e.currentTarget.value.length)}
          rows={Math.max(3, lines + 1)}
          onBlur={(e) => onText(e.currentTarget.value.trim())}
          className="sticky-words"
        />
      )}
      {strip}
      {!reading && !mark && (
        <span className="sticky-left">
          <Left most={IDEA_AT_MOST} used={used} />
        </span>
      )}
      {!reading && !mark && (
        <span className="sticky-tools">
          {note.text && (
            <button type="button" onClick={onFile} aria-label={t("ideaFile")} title={t("ideaFile")}>
              <Glyph name="folder-in" className="h-[13px] w-[13px]" />
            </button>
          )}
          {note.text && (
            <button type="button" onClick={onTask} aria-label={t("ideaTask")} title={t("ideaTask")}>
              <Glyph name="check-mark" className="h-[13px] w-[13px]" />
            </button>
          )}
          <button type="button" onClick={onDrop} aria-label={t("notesDrop")} title={t("notesDrop")}>
            ×
          </button>
        </span>
      )}
    </li>
  );
}

function Pin() {
  const id = useId();
  const held = useRef<HTMLSpanElement>(null);
  useLayoutEffect(() => {
    if (held.current) held.current.innerHTML = pinMarkup(`pin-${id}`);
  }, [id]);
  return <span ref={held} aria-hidden="true" className="contents" />;
}
