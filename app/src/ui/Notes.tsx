import { useEffect, useRef, useState } from "react";
import { locale, t } from "../locales";
import "../notes.css";
import {
  added,
  boardOf,
  dropped,
  type Note,
  rewritten,
  saidWhen,
  shapeOf,
  stampOf,
} from "../notes";

interface Props {
  title: string;
  body: string;
  reading: boolean;
  onWrite: (body: string) => void;
  onText: () => void;
}

export default function Notes({ title, body, reading, onWrite, onText }: Props) {
  const { notes } = boardOf(body);
  const [fresh, setFresh] = useState<number | null>(null);
  const newestFirst = notes.map((note, at) => ({ note, at })).reverse();

  return (
    <section aria-label={title} className="mx-auto w-full max-w-[880px] px-8 pt-6 pb-16">
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
          {newestFirst.map(({ note, at }) => (
            <Sticky
              key={`${at}-${note.heading}`}
              note={note}
              reading={reading}
              focused={fresh === at}
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
  reading: boolean;
  focused: boolean;
  onText: (text: string) => void;
  onDrop: () => void;
}

function Sticky({ note, reading, focused, onText, onDrop }: StickyProps) {
  const shape = shapeOf(note);
  const words = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    if (focused) words.current?.focus();
  }, [focused]);
  const lines = note.text.split("\n").length;

  return (
    <li
      className="sticky-note"
      data-hue={shape.hue}
      data-wide={shape.wide}
      data-pinned={shape.pinned ? "" : undefined}
      style={{ rotate: `${shape.tilt}deg`, marginTop: `${shape.drop * 8}px` }}
    >
      <span aria-hidden="true" className="sticky-hold" />
      <p className="sticky-when">{saidWhen(note.heading, locale())}</p>
      {reading ? (
        <p className="sticky-words">{note.text}</p>
      ) : (
        <textarea
          ref={words}
          aria-label={t("notesWords")}
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
