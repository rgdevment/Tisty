import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
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
  const wall = useRef<HTMLUListElement>(null);
  useSlides(wall, lifting);
  useEffect(() => {
    if (!lifting) return;
    const follow = (e: PointerEvent) => {
      const was = held.current;
      if (!was) return;
      const under = nearest(e.clientX, e.clientY, was.from);
      setLifted({
        ...was,
        dx: e.clientX - was.x,
        dy: e.clientY - was.y,
        over: under,
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
        <ul ref={wall} className="notes-wall">
          {newestFirst.map(({ note, at, nth }, shown) => (
            <Sticky
              key={`${note.heading}#${nth}#${note.text}`}
              stamp={`${note.heading}#${nth}#${note.text}`}
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

const SLIDES_FOR = 220;

// Each note glides from where it stood to where the new order puts it, the dropped one from where it was let go.
function useSlides(wall: React.RefObject<HTMLUListElement | null>, lifting: boolean) {
  const were = useRef(new Map<string, DOMRect>());
  useLayoutEffect(() => {
    const still = globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? true;
    const now = new Map<string, DOMRect>();
    for (const one of wall.current?.querySelectorAll<HTMLElement>("[data-key]") ?? []) {
      const box = one.getBoundingClientRect();
      const was = were.current.get(one.dataset.key ?? "");
      const dx = was ? was.left - box.left : 0;
      const dy = was ? was.top - box.top : 0;
      if (!lifting && !still && (dx || dy)) {
        one.animate?.([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "none" }], {
          duration: SLIDES_FOR,
          easing: "ease-out",
        });
      }
      now.set(one.dataset.key ?? "", box);
    }
    were.current = now;
  });
}

interface StickyProps {
  stamp: string;
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
  stamp,
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
      data-key={stamp}
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
      {shape.hold === "pin" ? <Pin /> : <span aria-hidden="true" className="sticky-hold" />}
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

function Pin() {
  const id = useId();
  const cap = `${id}-cap`;
  const neck = `${id}-neck`;
  const shade = (mix: string) => ({ stopColor: `color-mix(in srgb, var(--pin), ${mix})` });
  return (
    <svg aria-hidden="true" viewBox="0 0 24 44" className="sticky-pin">
      <defs>
        <radialGradient id={cap} cx="0.35" cy="0.3" r="0.8">
          <stop offset="0" style={shade("white 45%")} />
          <stop offset="0.45" style={{ stopColor: "var(--pin)" }} />
          <stop offset="1" style={shade("black 45%")} />
        </radialGradient>
        <linearGradient id={neck} x1="0" x2="1">
          <stop offset="0" style={shade("black 35%")} />
          <stop offset="0.35" style={shade("white 25%")} />
          <stop offset="1" style={shade("black 45%")} />
        </linearGradient>
      </defs>
      <line
        x1="12"
        y1="30"
        x2="14.5"
        y2="43"
        style={{ stroke: "var(--tisty-pin-needle)" }}
        strokeWidth="1.6"
        strokeLinecap="round"
      />
      <ellipse cx="12" cy="28.5" rx="9" ry="3.6" fill={`url(#${cap})`} />
      <path d="M8 9 L16 9 L14.2 27 L9.8 27 Z" fill={`url(#${neck})`} />
      <ellipse cx="12" cy="8" rx="8" ry="6.5" fill={`url(#${cap})`} />
      <ellipse cx="9.5" cy="5.5" rx="2.6" ry="1.6" fill="white" opacity="0.6" />
    </svg>
  );
}
