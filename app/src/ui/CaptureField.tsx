import { useEffect, useState } from "react";
import { read, type Edits, type Parsed, type Span, type Task } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Chips from "./Chips";
import Field, { type Mark } from "./Field";

interface Props {
  invite: string;
  onCapture: (text: string, edits: Edits) => Promise<Task>;
  onError: (message: string) => void;
}

const SETTLES = 120;

/** The parse trails the keystrokes; its offsets are only safe against the text it read (`of`). */
interface Read {
  of: string;
  seen: Parsed;
}

export default function CaptureField({ invite, onCapture, onError }: Props) {
  const [text, setText] = useState("");
  const [last, setLast] = useState<Read | null>(null);
  const [edits, setEdits] = useState<Edits>({});

  useEffect(() => {
    if (!text.trim()) {
      setLast(null);
      return;
    }
    const timer = setTimeout(() => {
      read(text)
        .then((seen) => setLast({ of: text, seen }))
        .catch(() => setLast(null));
    }, SETTLES);
    return () => clearTimeout(timer);
  }, [text]);

  return (
    <div className="w-full">
      <Field
        icon="＋"
        value={text}
        hint={invite}
        marks={last?.of === text ? marks(text, last.seen, edits) : []}
        onChange={(written) => {
          setText(written);
          // Edits answer to the sentence they were read from; a rewrite invalidates them.
          setEdits({});
        }}
        onSubmit={() =>
          onCapture(text, edits)
            .then(() => {
              setText("");
              setEdits({});
            })
            .catch((problem) => onError(saidPlainly(problem)))
        }
      />

      <div className="mt-2 px-1 text-[11.5px] text-faint">
        {last ? (
          <Chips seen={last.seen} edits={edits} onEdit={setEdits} empty={<Hint />} />
        ) : (
          <Hint />
        )}
      </div>
    </div>
  );
}

/** Removing a chip unmarks its words in the text too; an accepted offer renders like any other date. */
function marks(text: string, seen: Parsed, edits: Edits): Mark[] {
  const letters = Array.from(text);
  const gone = (span: Span) => {
    switch (span.mark) {
      case "date":
        return edits.noDate === true;
      case "deadline":
        return edits.noDeadline === true;
      case "list":
        return edits.noList === true;
      case "priority":
        return edits.noPriority === true;
      case "tag":
        return (edits.noTags ?? []).includes(
          letters.slice(span.from, span.to).join("").replace(/^#/, "").toLowerCase(),
        );
    }
  };

  // Picking another day does not put the words back, but they stopped deciding.
  const overruled = (span: Span) =>
    (span.mark === "date" && edits.date !== undefined) ||
    (span.mark === "deadline" && edits.deadline !== undefined);

  const taken = seen.spans
    .filter((span) => !gone(span))
    .map((span) => ({ span, offered: false, overruled: overruled(span) }));
  const offered = seen.offers.flatMap((offer) =>
    offer.spans.map((span) => ({ span, offered: !edits.date && !edits.deadline })),
  );
  return [...taken, ...offered];
}

function Hint() {
  return (
    <div className="flex gap-2.5 overflow-hidden whitespace-nowrap">
      <span>
        <Key>#</Key> {t("fieldTag")}
      </span>
      <span>
        <Key>@</Key> {t("fieldList")}
      </span>
      <span>
        <Key>!</Key> {t("fieldPriority")}
      </span>
      <span>{t("hintDates")}</span>
      <span>
        <Key>/</Key> {t("hintPick")}
      </span>
    </div>
  );
}

function Key({ children }: { children: string }) {
  return <code className="rounded bg-hover px-1.5 py-px text-[11px] text-soft">{children}</code>;
}
