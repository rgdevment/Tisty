import { type ReactNode, useEffect, useState } from "react";
import { complete, discard, docAway, docUnflag, type Filed, type List, type Task } from "../core";
import { useDesk } from "../desk";
import { dayOf, daysFrom, longAgo, whenLabel } from "../format";
import { fill, locale, t } from "../locales";
import { saidPlainly } from "../refusal";
import { NUDGE_AT, NUDGE_WEEKDAY, type Reviewing, reviewOf, reviewRemind } from "../reviewing";
import { dropWord } from "../tidying";
import { shown } from "../tongues";
import { useForgotten } from "../useForgotten";
import { usePinned } from "../usePinned";
import { MILD, RISKY } from "./Tidied";

const ROW = "flex flex-wrap items-center gap-x-3 gap-y-1 py-1 text-[12.5px]";
const TITLE = "min-w-0 cursor-pointer truncate text-left text-ink hover:underline";
const A_MONDAY = new Date(2026, 10, 16);

const weekdays = () =>
  Array.from({ length: 7 }, (_, n) => {
    const day = new Date(A_MONDAY);
    day.setDate(A_MONDAY.getDate() + n);
    return {
      n: n + 1,
      said: new Intl.DateTimeFormat(locale(), { weekday: "long" }).format(day),
    };
  });

export default function Review() {
  const { act, data, papers, setChosen, opening, dropDoc, lookPapers, setError, load } = useDesk();
  const [seen, setSeen] = useState<Reviewing | null>(null);
  useEffect(() => {
    let current = true;
    reviewOf()
      .then((found) => current && setSeen(found))
      .catch(() => current && setSeen(null));
    return () => {
      current = false;
    };
  }, [data]);
  const pinned = usePinned(papers);
  const docs = shown(papers.docs);
  const forgotten = useForgotten(docs, true);
  const marked = docs.filter((one) => one.flagged && !one.away && !one.pageOf);
  const unlisted = seen?.unlisted ?? [];
  const ideas = pinned?.ideas ?? 0;
  const failed = (e: unknown) => setError(saidPlainly(e));
  const putAway = (one: Filed) => docAway(one.id, true).then(lookPapers).catch(failed);
  const openDoc = (one: Filed) => setChosen({ named: "docs", doc: one.file });
  const folderOf = (one: Filed) =>
    papers.folders.find((at) => at.id === one.folder)?.name ?? t("unfiled");

  return (
    <main className="scroller min-w-0 flex-1 px-7 pt-5 pb-8">
      <h1 className="text-[21px] font-semibold">{t("weekly")}</h1>
      <p className="mt-1 text-[12.5px] text-faint">{t("weeklyLead")}</p>

      <Stop
        at={1}
        title={t("weeklyToDecide")}
        hint={t("weeklyToDecideHint")}
        many={unlisted.length + ideas}
      >
        {unlisted.map((task) => (
          <TaskRow
            key={task.id}
            task={task}
            onOpen={() => opening(task)}
            onDone={() => act(complete(task.id))}
            onDrop={() => act(discard(task.id))}
          />
        ))}
        {pinned && ideas > 0 && (
          <div className={ROW}>
            <button
              type="button"
              onClick={() => setChosen({ named: "docs", doc: pinned.file })}
              className={MILD}
            >
              {fill("weeklyIdeas", String(ideas))}
            </button>
          </div>
        )}
      </Stop>

      <Stop at={2} title={t("weeklyMarked")} hint={t("weeklyMarkedHint")} many={marked.length}>
        {marked.map((one) => (
          <div key={one.id} className={ROW}>
            <button type="button" onClick={() => openDoc(one)} className={TITLE}>
              {one.title || t("untitledDoc")}
            </button>
            <span className="min-w-0 truncate text-soft">{one.flagged?.said}</span>
            <span className="ml-auto flex gap-2">
              <button type="button" onClick={() => void putAway(one)} className={MILD}>
                {t("putAway")}
              </button>
              {!one.locked && (
                <button type="button" onClick={() => dropDoc(one)} className={RISKY}>
                  {t(dropWord(one.flagged?.reason))}
                </button>
              )}
              <button
                type="button"
                onClick={() => docUnflag(one.id).then(lookPapers).catch(failed)}
                className={MILD}
              >
                {t("unflagIt")}
              </button>
            </span>
          </div>
        ))}
      </Stop>

      <Stop
        at={3}
        title={t("forgottenHeading")}
        hint={t("weeklyForgottenHint")}
        many={forgotten.length}
      >
        {forgotten.map(({ doc, last }) => (
          <div key={doc.id} className={ROW}>
            <button type="button" onClick={() => openDoc(doc)} className={TITLE}>
              {doc.title || t("untitledDoc")}
            </button>
            <span className="text-[11.5px] text-faint">
              {[folderOf(doc), longAgo(last)].join(" · ")}
            </span>
            <span className="ml-auto">
              <button type="button" onClick={() => void putAway(doc)} className={MILD}>
                {t("putAway")}
              </button>
            </span>
          </div>
        ))}
      </Stop>

      {seen?.routine ? (
        <footer className="mt-8 flex flex-wrap items-center gap-3">
          {seen.routine.date && daysFrom(seen.routine.date.at) <= 0 && (
            <button
              type="button"
              onClick={() => seen.routine && act(complete(seen.routine.id))}
              className={MILD}
            >
              {t("weeklyFinish")}
            </button>
          )}
          {seen.routine.date && (
            <span className="text-[12.5px] text-faint">
              {fill("weeklyDue", dayOf(seen.routine.date.at))}
            </span>
          )}
        </footer>
      ) : (
        seen && <Nudging lists={data.every ?? data.lists} onMade={load} onError={failed} />
      )}
    </main>
  );
}

function Stop({
  at,
  title,
  hint,
  many,
  children,
}: {
  at: number;
  title: string;
  hint: string;
  many: number;
  children: ReactNode;
}) {
  return (
    <section aria-label={title} className="mt-6">
      <h2 className="flex items-center gap-2 text-[12.5px] font-semibold">
        <span className="text-faint tabular-nums">{at}</span>
        {title}
        {many === 0 ? (
          <span role="img" aria-label={t("weeklyClear")} className="text-hue-teal">
            ✓
          </span>
        ) : (
          <span className="text-[11.5px] font-normal text-faint tabular-nums">{many}</span>
        )}
      </h2>
      <p className="text-[11.5px] text-faint">{many === 0 ? t("weeklyEmpty") : hint}</p>
      {many > 0 && <div className="mt-1">{children}</div>}
    </section>
  );
}

function TaskRow({
  task,
  onOpen,
  onDone,
  onDrop,
}: {
  task: Task;
  onOpen: () => void;
  onDone: () => void;
  onDrop: () => void;
}) {
  return (
    <div className={ROW}>
      <button type="button" onClick={onOpen} className={TITLE}>
        {task.title}
      </button>
      {task.date && <span className="text-[11.5px] text-faint">{whenLabel(task.date)}</span>}
      <span className="ml-auto flex gap-2">
        <button type="button" onClick={onOpen} className={MILD}>
          {t("weeklyOpen")}
        </button>
        <button type="button" onClick={onDone} className={MILD}>
          {t("markDone")}
        </button>
        <button type="button" onClick={onDrop} className={MILD}>
          {t("discardIt")}
        </button>
      </span>
    </div>
  );
}

function Nudging({
  lists,
  onMade,
  onError,
}: {
  lists: List[];
  onMade: () => void;
  onError: (e: unknown) => void;
}) {
  const open = lists.filter((one) => !one.archived);
  const [weekday, setWeekday] = useState(NUDGE_WEEKDAY);
  const [at, setAt] = useState(NUDGE_AT);
  const [list, setList] = useState("");
  const into = list || open[0]?.id || "";
  const [hour, minute] = at.split(":").map(Number);
  const made = () =>
    reviewRemind({ title: t("weekly"), tag: t("weeklyTag"), list: into, weekday, hour, minute })
      .then(onMade)
      .catch(onError);

  return (
    <section aria-label={t("weeklyNudge")} className="mt-8 rounded-[10px] border border-line p-4">
      <p className="text-[12.5px] text-soft">{t("weeklyNudgeLead")}</p>
      <div className="mt-3 flex flex-wrap items-end gap-3 text-[12.5px]">
        <label className="flex flex-col gap-1 text-faint">
          {t("weeklyDay")}
          <select
            value={weekday}
            onChange={(e) => setWeekday(Number(e.target.value))}
            className="rounded-md border border-line bg-sheet px-2 py-1 text-ink"
          >
            {weekdays().map((one) => (
              <option key={one.n} value={one.n}>
                {one.said}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1 text-faint">
          {t("weeklyHour")}
          <input
            type="time"
            value={at}
            onChange={(e) => setAt(e.target.value || NUDGE_AT)}
            className="rounded-md border border-line bg-sheet px-2 py-1 text-ink"
          />
        </label>
        <label className="flex flex-col gap-1 text-faint">
          {t("weeklyList")}
          <select
            value={into}
            onChange={(e) => setList(e.target.value)}
            className="rounded-md border border-line bg-sheet px-2 py-1 text-ink"
          >
            {open.map((one) => (
              <option key={one.id} value={one.id}>
                {one.name}
              </option>
            ))}
          </select>
        </label>
        <button type="button" disabled={!into} onClick={() => void made()} className={MILD}>
          {t("weeklyNudge")}
        </button>
      </div>
    </section>
  );
}
