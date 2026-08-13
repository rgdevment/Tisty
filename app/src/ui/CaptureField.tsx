import { useEffect, useState } from "react";
import { read, type Counted, type Edits, type List, type Parsed, type Span, type Task } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Calendar from "./Calendar";
import Chips from "./Chips";
import Field, { type Mark } from "./Field";
import SlashMenu from "./SlashMenu";

interface Props {
  invite: string;
  lists: List[];
  tags: Counted[];
  onCapture: (text: string, edits: Edits) => Promise<Task>;
  onError: (message: string) => void;
}

const CALLED = /(^|\s)([/@#])(\S*)$/;
const OPENS: Record<string, "list" | "tag" | null> = { "/": null, "@": "list", "#": "tag" };

const SETTLES = 120;

interface Read {
  of: string;
  seen: Parsed;
}

export default function CaptureField({ invite, lists, tags, onCapture, onError }: Props) {
  const [text, setText] = useState("");
  const [last, setLast] = useState<Read | null>(null);
  const [edits, setEdits] = useState<Edits>({});
  const [picking, setPicking] = useState<"date" | "deadline" | null>(null);
  const [dismissed, setDismissed] = useState<number | null>(null);

  const asked = picking === null ? CALLED.exec(text) : null;
  const before = asked ? text.slice(0, text.length - asked[3].length - 1) : text;
  const called = asked && before.length !== dismissed ? asked : null;
  const rewritten = (typed: string) => {
    setText(typed);
    setEdits(({ date, deadline, takeOffer }) =>
      typed.trim() ? { date, deadline, takeOffer } : {},
    );
  };

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
        onChange={(typed) => {
          rewritten(typed);
          if (!CALLED.test(typed)) setDismissed(null);
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

      <div className="relative">
        {called && (
          <SlashMenu
            from={OPENS[called[2]]}
            query={called[3]}
            lists={lists}
            tags={tags}
            onDate={(field) => {
              setText(before);
              setPicking(field);
            }}
            onInsert={(marker) => rewritten(`${before}${marker} `)}
            onClose={() => setDismissed(before.length)}
          />
        )}
        {picking && (
          <Calendar
            onPick={(iso) => {
              setEdits({ ...edits, [picking]: iso, [picking === "date" ? "noDate" : "noDeadline"]: false });
              setPicking(null);
            }}
            onClear={() => setPicking(null)}
            onClose={() => setPicking(null)}
          />
        )}
      </div>

      <div className="mt-2 px-1 text-[11.5px] text-faint">
        {last ? (
          <>
            <Chips seen={last.seen} edits={edits} onEdit={setEdits} empty={null} />
            <Hint />
          </>
        ) : (
          <Hint />
        )}
      </div>
    </div>
  );
}

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
      case "tag": {
        const written = letters.slice(span.from, span.to).join("");
        return (edits.noTags ?? []).some((one) => alike(one, written));
      }
    }
  };

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

const alike = (tag: string, written: string): boolean =>
  written
    .replace(/^#/, "")
    .toLowerCase()
    .replace(/[ _]/g, "-")
    .replace(/[^\p{L}\p{N}-]/gu, "") === tag;

function Hint() {
  return (
    <div className="flex flex-wrap gap-x-2.5 gap-y-1">
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
        <Key>↻</Key> {t("hintRepeat")}
      </span>
      <span>
        <Key>/</Key> {t("hintPick")}
      </span>
    </div>
  );
}

function Key({ children }: { children: string }) {
  return <code className="rounded bg-hover px-1.5 py-px text-[11px] text-soft">{children}</code>;
}
