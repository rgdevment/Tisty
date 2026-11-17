import { useCallback, useEffect, useRef, useState } from "react";
import {
  type DayAt,
  type Diary,
  diaryAgents,
  diaryDayWords,
  diaryLaid,
  diaryNewest,
  diaryOf,
  diaryRewrite,
  diaryUnstick,
  type Today,
} from "../diary";
import { t } from "../locales";
import { sure } from "../sure";
import DiaryComposer from "./DiaryComposer";
import DiaryHead from "./DiaryHead";
import Page from "./Page";
import { useRestrips } from "./useRestrips";

interface Props {
  file: string;
  body: string;
  title: string;
  book?: string;
  fresh: number;
  writable: boolean;
  onDoc: (file: string) => void;
  onTask: (id: string) => void;
  onBefore: () => Promise<unknown>;
  onWritten: (today: Today) => void;
  onEdit: () => void;
  onError: (problem: unknown) => void;
}

type Opened = { file: string; at: DayAt; was: string };

type Typed = { words: string; start: number; end: number; focused: boolean };

const boxOf = (page: Document | undefined) =>
  page?.querySelector<HTMLTextAreaElement>(".diary-box textarea") ?? null;

// The month is drawn again whenever the log or the file moves, so what is typed lives here, not in the box.
const useDayBox = () => {
  const typed = useRef<Typed | null>(null);
  const shown = useRef<{ page?: Document; body: string }>({ body: "" });
  const heard = useRef<WeakSet<Document> | null>(null);
  const note = (event: Event) => {
    const box = event.target as HTMLTextAreaElement | null;
    if (!box?.matches?.(".diary-box textarea")) return;
    const said = { words: box.value, start: box.selectionStart, end: box.selectionEnd };
    if (event.type !== "focusout") {
      typed.current = { ...said, focused: true };
      return;
    }
    // A box taken away by a new drawing loses its focus too, and the new one should take it back.
    setTimeout(() => {
      if (boxOf(shown.current.page) === box) typed.current = { ...said, focused: false };
    }, 0);
  };
  const putBack = (page: Document) => {
    const box = boxOf(page);
    if (!box) return;
    const now = typed.current;
    if (now) box.value = now.words;
    if (now && !now.focused) return;
    box.focus();
    box.setSelectionRange(now?.start ?? box.value.length, now?.end ?? box.value.length);
  };
  const onShown = (frame: Window | null, body?: string) => {
    const page = frame?.document;
    shown.current = { page, body: body ?? "" };
    if (!page) return;
    heard.current ??= new WeakSet();
    if (!heard.current.has(page)) {
      heard.current.add(page);
      for (const kind of ["input", "keyup", "mouseup", "focusout"])
        page.addEventListener(kind, note);
    }
    putBack(page);
  };
  return { typed, shown, onShown };
};

// A year opens on its newest month, and this month's page opens on today.
export default function DiaryView(props: Readonly<Props>) {
  const { file, body, title, book, fresh, writable, onError } = props;
  const [kept, setKept] = useState<Diary | null>(null);
  useEffect(() => {
    if (fresh < 0) return;
    let live = true;
    diaryOf()
      .then((now) => live && setKept(now))
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [fresh]);
  const paged = Boolean(book);
  const leave = useRef(props.onDoc);
  leave.current = props.onDoc;
  useEffect(() => {
    if (paged) return;
    let live = true;
    diaryNewest(file)
      .then((page) => {
        if (live && page) leave.current(page);
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [file, paged]);
  const restrips = useRestrips(paged ? file : undefined, body, fresh);
  const [held, setOpened] = useState<Opened | null>(null);
  const opened = held?.file === file ? held : null;
  const { typed, shown, onShown } = useDayBox();
  const keeping = useRef(false);
  const at = writable ? opened?.at : undefined;
  const draw = useCallback((text: string) => diaryLaid(file, text, at), [file, at]);
  const year = book?.match(/[0-9]{4}/)?.[0] ?? "";
  const thisMonth = paged && kept?.page === file;
  const turn = () => {
    if (!kept) return;
    const open = !kept.open;
    diaryAgents(open)
      .then(() => setKept({ ...kept, open }))
      .catch(onError);
  };

  const words = () => typed.current?.words ?? opened?.was;
  // A box whose day is gone from the page holds nothing anyone can still keep.
  const changed = () =>
    opened !== null && words() !== opened.was && boxOf(shown.current.page) !== null;
  const close = () => {
    typed.current = null;
    setOpened(null);
  };
  const openDay = (day: DayAt) => {
    if (changed()) return;
    diaryDayWords(body, day)
      .then((was) => {
        if (was === null) return;
        typed.current = null;
        setOpened({ file, at: day, was });
      })
      .catch(onError);
  };
  const keepDay = () => {
    const said = words();
    if (!opened || said === undefined || keeping.current) return;
    if (said === opened.was) return close();
    keeping.current = true;
    props
      .onBefore()
      .catch(() => {})
      .then(() => diaryRewrite(file, opened.at, opened.was, said))
      .then((today) => {
        close();
        props.onWritten(today);
      })
      .catch(onError)
      .finally(() => {
        keeping.current = false;
      });
  };
  const unstick = (day: DayAt, note: number) => {
    const seen = shown.current.body;
    sure(t("diaryUnstickSure"))
      .then(async (yes) => {
        if (!yes) return;
        await props.onBefore().catch(() => {});
        props.onWritten(await diaryUnstick(file, day, note, seen));
      })
      .catch(onError);
  };
  const ways = writable
    ? {
        onDay: openDay,
        onDayKeep: keepDay,
        onDayCancel: close,
        onUnstick: unstick,
        onAway: () => {
          if (opened && !changed()) close();
        },
      }
    : {};

  return (
    <div className="mx-auto flex min-h-0 w-full max-w-[720px] flex-1 flex-col">
      <div className="flex flex-col gap-4 px-4 pt-6 pb-5">
        <DiaryHead
          over={paged ? `${title} ${year}`.trim() : ""}
          big={thisMonth && kept ? kept.day : title}
          open={kept ? kept.open : null}
          onTurn={turn}
          onEdit={paged ? props.onEdit : undefined}
        />
        <DiaryComposer onBefore={props.onBefore} onWritten={props.onWritten} onError={onError} />
      </div>
      {paged && (
        <Page
          body={body}
          draw={draw}
          again={String(restrips)}
          label={title}
          onDoc={props.onDoc}
          onTask={props.onTask}
          onShown={onShown}
          {...ways}
        />
      )}
    </div>
  );
}
