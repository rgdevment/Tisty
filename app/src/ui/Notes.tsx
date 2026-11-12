import { useEffect, useRef, useState } from "react";
import { locale, t } from "../locales";
import "../notes.css";
import {
  added,
  boardOf,
  dropped,
  type Note,
  ordered,
  rewritten,
  saidWhen,
  shapeOf,
  shifted,
  stampOf,
} from "../notes";

interface Props {
  title: string;
  body: string;
  reading: boolean;
  onWrite: (body: string) => void;
  onText: () => void;
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

export default function Notes({ title, body, reading, onWrite, onText }: Props) {
  const { notes } = boardOf(body);
  const [fresh, setFresh] = useState<number | null>(null);
  const [lifted, setLifted] = useState<Lifted | null>(null);
  const newestFirst = notes
    .map((note, at) => ({
      note,
      at,
      nth: notes.slice(0, at).filter((one) => one.heading === note.heading).length,
    }))
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
  useEffect(() => {
    if (!lifting) return;
    const follow = (e: PointerEvent) => {
      const was = held.current;
      if (!was) return;
      const under = document
        .elementFromPoint(e.clientX, e.clientY)
        ?.closest<HTMLElement>("[data-shown]");
      setLifted({
        ...was,
        dx: e.clientX - was.x,
        dy: e.clientY - was.y,
        over: under ? Number(under.dataset.shown) : null,
      });
    };
    const drop = () => {
      const was = held.current;
      setLifted(null);
      if (was && was.over !== null && Math.hypot(was.dx, was.dy) > LIFTS_PAST) {
        settle.current(was.from, was.over);
      }
    };
    globalThis.addEventListener("pointermove", follow);
    globalThis.addEventListener("pointerup", drop);
    globalThis.addEventListener("pointercancel", drop);
    return () => {
      globalThis.removeEventListener("pointermove", follow);
      globalThis.removeEventListener("pointerup", drop);
      globalThis.removeEventListener("pointercancel", drop);
    };
  }, [lifting]);

  return (
    <section aria-label={title} className="w-full px-8 pt-6 pb-16">
      <div className="flex items-center gap-2">
        <h1 className="min-w-0 flex-1 truncate text-[21px] font-semibold">{title}</h1>
        {!reading && (
          <button
            type="button"
            onClick={() => {
              setFresh(notes.length);
              onWrite(added(body, stampOf(new Date())));
            }}
            className="rounded-md px-2 py-1 text-[12.5px] text-accent hover:bg-hover"
          >
            {t("notesAdd")}
          </button>
        )}
        <button
          type="button"
          onClick={onText}
          className="rounded-md px-2 py-1 text-[12.5px] text-soft hover:bg-hover"
        >
          {t("notesAsText")}
        </button>
      </div>
      {notes.length === 0 ? (
        <p className="pt-6 text-[12.5px] text-faint">{t("notesNone")}</p>
      ) : (
        <ul className="notes-wall">
          {newestFirst.map(({ note, at, nth }, shown) => (
            <Sticky
              key={`${note.heading}#${nth}#${note.text}`}
              note={note}
              nth={nth}
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
            />
          ))}
        </ul>
      )}
    </section>
  );
}

interface StickyProps {
  note: Note;
  nth: number;
  shown: number;
  reading: boolean;
  focused: boolean;
  lift: { dx: number; dy: number } | null;
  target: boolean;
  onLift: (x: number, y: number) => void;
  onMove: (by: number) => void;
  onText: (text: string) => void;
  onDrop: () => void;
}

const EARLIER = new Set(["ArrowUp", "ArrowLeft"]);
const LATER = new Set(["ArrowDown", "ArrowRight"]);

function Sticky({
  note,
  nth,
  shown,
  reading,
  focused,
  lift,
  target,
  onLift,
  onMove,
  onText,
  onDrop,
}: StickyProps) {
  const shape = shapeOf(note.heading, nth);
  const words = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    if (focused) words.current?.focus();
  }, [focused]);
  const lines = note.text.split("\n").length;
  const nudge = shape.nudge * 6;

  return (
    <li
      data-shown={shown}
      className="sticky-note"
      data-hue={shape.hue}
      data-wide={shape.wide}
      data-hold={shape.hold}
      data-pin={shape.pin}
      data-lifted={lift ? "" : undefined}
      data-target={target ? "" : undefined}
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
          "--angle": `${shape.angle}deg`,
          "--along": `${shape.along}%`,
        } as React.CSSProperties
      }
    >
      <span aria-hidden="true" className="sticky-hold" />
      <p className="sticky-when">{saidWhen(note.heading, locale())}</p>
      {reading ? (
        <p className="sticky-words">{note.text}</p>
      ) : (
        <textarea
          ref={words}
          aria-label={t("notesWords")}
          aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown"
          defaultValue={note.text}
          rows={Math.max(3, lines + 1)}
          onBlur={(e) => onText(e.currentTarget.value.trim())}
          className="sticky-words"
        />
      )}
      {!reading && (
        <button type="button" onClick={onDrop} className="sticky-drop" aria-label={t("notesDrop")}>
          ×
        </button>
      )}
    </li>
  );
}
