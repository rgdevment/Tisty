import { useEffect, useMemo, useState } from "react";
import type { Series, Turn } from "../core";
import { taskSeries } from "../core";
import { cadence } from "../format";
import { fill, locale, t } from "../locales";

interface Props {
  task: string;
  onError?: (problem: unknown) => void;
}

type Mark = "kept" | "gap" | "given" | "open";

export default function Routine({ task, onError }: Props) {
  const [told, setTold] = useState<Series | null>(null);

  useEffect(() => {
    let alive = true;
    setTold(null);
    taskSeries(task)
      .then((series) => {
        if (alive) setTold(series);
      })
      .catch((problem) => onError?.(problem));
    return () => {
      alive = false;
    };
  }, [task, onError]);

  const days = useMemo(() => (told ? laid(told) : []), [told]);
  const hours = useMemo(() => (told ? clocked(told) : null), [told]);

  if (!told) return null;

  const late = told.turns
    .map((turn) => turn.late)
    .filter((one): one is number => one !== undefined && one > 0);
  const average = late.length
    ? Math.round((late.reduce((sum, one) => sum + one, 0) / late.length) * 10) / 10
    : 0;

  return (
    <div>
      <div className="flex flex-wrap gap-x-6 gap-y-3">
        <Fact
          big={`${told.kept}/${told.owed}`}
          small={t("routineKept")}
          aside={told.owed > 0 ? `${Math.round((told.kept / told.owed) * 100)} %` : undefined}
        />
        <Fact big={String(told.streak)} small={t("routineStreak")} />
        <Fact big={String(told.longest)} small={t("routineLongest")} />
        {told.measurable && <Fact big={String(told.skipped)} small={t("routineGaps")} />}
        {told.dropped > 0 && <Fact big={String(told.dropped)} small={t("routineDropped")} />}
        {hours && <Fact big={hours.usual} small={t("routineUsual")} />}
      </div>

      {told.repeat && (
        <p className="mt-3 text-[12px] text-faint">
          ↻ {cadence(told.repeat)}
          {" · "}
          {told.repeat.until ? fill("routineUntil", dated(told.repeat.until)) : t("routineEndless")}
          {average > 0 && ` · ${fill("routineLate", `${average} d`)}`}
        </p>
      )}

      <Line label={t("routineTurns")} />
      <div className="flex flex-wrap gap-[3px]">
        {days.map((day) => (
          <span
            key={day.key}
            title={`${day.when}${day.told ? ` · ${t("routineKeyTold")}` : ""}`}
            className={`h-3 w-3 rounded-[3px] ${paint(day.mark)} ${
              day.told ? "ring-2 ring-high ring-offset-1 ring-offset-bg" : ""
            }`}
          />
        ))}
      </div>
      <div className="mt-2.5 flex flex-wrap gap-x-4 gap-y-1 text-[11px] text-faint">
        <Key mark="kept" word={t("routineKeyKept")} />
        {told.measurable && <Key mark="gap" word={t("routineKeyGap")} />}
        {told.dropped > 0 && <Key mark="given" word={t("routineKeyGiven")} />}
        <Key mark="open" word={t("routineKeyOpen")} />
      </div>

      {hours && (
        <>
          <Line label={t("routineWhen")} />
          <div className="flex h-12 items-end gap-[2px]" role="img" aria-label={hours.said}>
            {hours.bars.map((one) => (
              <span
                key={one.hour}
                title={`${String(one.hour).padStart(2, "0")}:00 · ${one.many}`}
                style={{ height: `${Math.max(one.tall, one.tall > 0 ? 6 : 2)}%` }}
                className={`flex-1 rounded-t-[3px] bg-accent ${
                  one.tall > 0 ? "opacity-70" : "opacity-15"
                }`}
              />
            ))}
          </div>
          <div className="mt-1 flex justify-between text-[11px] tabular-nums text-faint">
            <span>{hours.from}</span>
            <span>{hours.usual}</span>
            <span>{hours.to}</span>
          </div>
          <p className="mt-2 text-[11px] leading-relaxed text-faint">{t("routineZone")}</p>
        </>
      )}

      <Line label={t("routineHoles")} />
      {!told.measurable ? (
        <p className="text-[12px] leading-relaxed text-faint">{t("routineUnmeasured")}</p>
      ) : told.skipped === 0 ? (
        <p className="text-[12px] text-faint">{t("routineNoHoles")}</p>
      ) : (
        <ul className="flex flex-wrap gap-x-3 gap-y-1 text-[12px] text-soft">
          {told.turns.flatMap((turn) =>
            (turn.gaps ?? []).map((gap) => (
              <li key={gap} className="tabular-nums">
                {dated(gap)}
              </li>
            )),
          )}
        </ul>
      )}
    </div>
  );
}

function Fact({ big, small, aside }: { big: string; small: string; aside?: string }) {
  return (
    <div>
      <b className="block text-[19px] leading-tight font-bold tracking-tight tabular-nums">
        {big}
        {aside && <span className="ml-1.5 text-[12px] font-normal text-faint">{aside}</span>}
      </b>
      <span className="text-[11.5px] text-faint">{small}</span>
    </div>
  );
}

function Line({ label }: { label: string }) {
  return (
    <div className="mt-5 mb-2 flex items-center gap-2.5 text-[11px] font-semibold tracking-[0.06em] text-faint uppercase">
      <span>{label}</span>
      <span className="h-px flex-1 bg-hair" />
    </div>
  );
}

function Key({ mark, word }: { mark: Mark; word: string }) {
  return (
    <span className="flex items-center gap-1.5">
      <span className={`h-3 w-3 rounded-[3px] ${paint(mark)}`} />
      {word}
    </span>
  );
}

const paint = (mark: Mark): string =>
  mark === "kept"
    ? "bg-accent"
    : mark === "gap"
      ? "border-[1.5px] border-urgent bg-transparent"
      : mark === "given"
        ? "border-[1.5px] border-faint bg-transparent"
        : "bg-hair";

interface Day {
  key: string;
  when: string;
  mark: Mark;
  told: boolean;
}

function laid(told: Series): Day[] {
  const days: Day[] = [];
  for (const turn of told.turns) {
    for (const gap of turn.gaps ?? []) {
      days.push({ key: `gap-${gap}`, when: gap, mark: "gap", told: false });
    }
    days.push({
      key: turn.id,
      when: turn.due?.at?.slice(0, 10) ?? "",
      mark: mark(turn),
      told: turn.told === true,
    });
  }
  return days;
}

const mark = (turn: Turn): Mark =>
  turn.status === "done" ? "kept" : turn.status === "dropped" ? "given" : "open";

const dated = (day: string): string => {
  const at = new Date(`${day}T12:00:00`);
  if (Number.isNaN(at.getTime())) return day;
  return new Intl.DateTimeFormat(locale(), {
    weekday: "short",
    day: "numeric",
    month: "short",
  }).format(at);
};

function clocked(told: Series) {
  const counts = new Array(24).fill(0) as number[];
  let seen = 0;
  for (const turn of told.turns) {
    if (!turn.closed) continue;
    const at = new Date(turn.closed);
    if (Number.isNaN(at.getTime())) continue;
    counts[at.getHours()] += 1;
    seen += 1;
  }
  if (!seen) return null;

  const most = Math.max(...counts);
  const peak = counts.indexOf(most);
  const busy = counts.map((many, hour) => ({ many, hour })).filter((one) => one.many > 0);
  const from = Math.max(0, Math.min(...busy.map((one) => one.hour)) - 1);
  const to = Math.min(23, Math.max(...busy.map((one) => one.hour)) + 1);
  return {
    bars: counts
      .map((many, hour) => ({ hour, many, tall: (many / most) * 100 }))
      .slice(from, to + 1),
    from: `${String(from).padStart(2, "0")}`,
    to: `${String(to).padStart(2, "0")}`,
    usual: `${String(peak).padStart(2, "0")}:00`,
    said: fill("routineWhen", String(seen)),
  };
}
