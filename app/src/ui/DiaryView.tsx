import { useCallback, useEffect, useState } from "react";
import { docDrawn } from "../core";
import { type Diary, diaryAgents, diaryLaid, diaryOf, type Today } from "../diary";
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
  onDoc: (file: string) => void;
  onTask: (id: string) => void;
  onBefore: () => Promise<unknown>;
  onWritten: (today: Today) => void;
  onEdit: () => void;
  onError: (problem: unknown) => void;
}

const named = (date: Date): string => {
  const weekday = new Intl.DateTimeFormat(undefined, { weekday: "long" }).format(date);
  return `${weekday.charAt(0).toUpperCase()}${weekday.slice(1)} ${date.getDate()}`;
};

// A month opens on today when it is this month's page, and on its own name when it is another.
export default function DiaryView(props: Readonly<Props>) {
  const { file, body, title, book, fresh, onError } = props;
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
  const restrips = useRestrips(paged ? file : undefined, body, fresh);
  const draw = useCallback(
    (text: string) => (paged ? diaryLaid(file, text) : docDrawn(text)),
    [file, paged],
  );
  const year = book?.match(/[0-9]{4}/)?.[0] ?? "";
  const thisMonth = paged && kept?.page === file;
  const turn = () => {
    if (!kept) return;
    const open = !kept.open;
    diaryAgents(open)
      .then(() => setKept({ ...kept, open }))
      .catch(onError);
  };

  return (
    <div className="mx-auto flex min-h-0 w-full max-w-[720px] flex-1 flex-col">
      <div className="flex flex-col gap-4 px-4 pt-6 pb-5">
        <DiaryHead
          over={paged ? `${title} ${year}`.trim() : ""}
          big={thisMonth ? named(new Date()) : title}
          open={kept ? kept.open : null}
          onTurn={turn}
          onEdit={paged ? props.onEdit : undefined}
        />
        <DiaryComposer onBefore={props.onBefore} onWritten={props.onWritten} onError={onError} />
      </div>
      <Page
        body={body}
        draw={draw}
        again={String(restrips)}
        label={title}
        onDoc={props.onDoc}
        onTask={props.onTask}
      />
    </div>
  );
}
