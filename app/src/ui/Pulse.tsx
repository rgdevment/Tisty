import type { Coming, Habit, List } from "../core";
import { t } from "../locales";
import { QUADRANTS, said, tint } from "../quadrants";
import Ahead, { WEEK } from "./Ahead";

interface Props {
  apart: string;
  counts: Record<string, number>;
  lists: List[];
  ahead: Coming[];
  routines: Habit[];
  papers: number;
  onList: (id: string) => void;
  onQuadrants: () => void;
  onOpen: (task: string) => void;
}

export default function Pulse({
  apart,
  counts,
  lists,
  ahead,
  routines,
  papers,
  onList,
  onQuadrants,
  onOpen,
}: Props) {
  const held = lists
    .filter((one) => counts[one.id])
    .sort((a, b) => (counts[b.id] ?? 0) - (counts[a.id] ?? 0));

  return (
    <aside
      className={`scroller absolute top-11 bottom-3 w-[300px] flex-col gap-4 rounded-[10px] border border-hair bg-panel px-3.5 pt-4 pb-5 shadow-lift ${apart}`}
      style={{ right: 12 }}
    >
      <div>
        <Cap said={t("theDay")} />
        <dl className="grid grid-cols-3 gap-1.5">
          <Count many={counts.overdue ?? 0} said={t("pulseLate")} tone="text-urgent" />
          <Count many={counts.dueToday ?? 0} said={t("pulseToday")} tone="text-accent" />
          <Count many={counts.upcoming ?? 0} said={t("pulseAhead")} />
        </dl>
      </div>

      <div>
        <Cap said={t("upcoming")} />
        <Ahead coming={ahead} routines={routines} days={WEEK} onOpen={onOpen} />
      </div>

      <div>
        <Cap said={t("quadrants")} />
        <div className="grid grid-cols-2 gap-1.5">
          {QUADRANTS.map((one) => (
            <button
              key={one}
              type="button"
              onClick={onQuadrants}
              className="flex items-baseline gap-2 rounded-[10px] border border-hair px-2 py-1 text-left hover:bg-hover"
            >
              <span className="min-w-0 truncate text-[11.5px] text-soft">{said(one)}</span>
              <b className={`ml-auto text-[12.5px] tabular-nums ${tint(one)}`}>
                {counts[one] ?? 0}
              </b>
            </button>
          ))}
        </div>
        {counts.quadrants ? (
          <button
            type="button"
            onClick={onQuadrants}
            className="mt-1 flex w-full items-baseline gap-2 text-left text-[11.5px] text-faint hover:text-ink"
          >
            <span>{t("noPriority")}</span>
            <span className="ml-auto tabular-nums">{counts.quadrants}</span>
          </button>
        ) : null}
      </div>

      {held.length > 0 && (
        <div>
          <Cap said={t("lists")} />
          {held.map((one) => (
            <Line
              key={one.id}
              said={one.name}
              many={counts[one.id] ?? 0}
              onPick={() => onList(one.id)}
            />
          ))}
        </div>
      )}

      <div>
        <Cap said={t("pulseElse")} />
        <Line said={t("sliceAll")} many={counts.all ?? 0} quiet />
        <Line said={t("noList")} many={counts.inbox ?? 0} quiet />
        <Line said={t("pulseUndated")} many={counts.undated ?? 0} quiet />
        <Line said={t("repeating")} many={counts.routines ?? 0} quiet />
        <Line said={t("archived")} many={counts.archive ?? 0} quiet />
        <Line said={t("docs")} many={papers} quiet />
      </div>
    </aside>
  );
}

function Cap({ said }: { said: string }) {
  return (
    <p className="mb-1.5 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase">
      {said}
    </p>
  );
}

function Line({
  said,
  many,
  onPick,
  quiet,
}: {
  said: string;
  many: number;
  onPick?: () => void;
  quiet?: boolean;
}) {
  const shown = (
    <>
      <span className="min-w-0 truncate">{said}</span>
      <span className="ml-auto shrink-0 tabular-nums text-faint">{many}</span>
    </>
  );
  if (quiet || !onPick) {
    return <p className="flex items-baseline gap-2 py-px text-[11.5px] text-faint">{shown}</p>;
  }
  return (
    <button
      type="button"
      onClick={onPick}
      className="flex w-full items-baseline gap-2 py-px text-left text-[11.5px] text-soft hover:text-ink"
    >
      {shown}
    </button>
  );
}

function Count({ many, said, tone }: { many: number; said: string; tone?: string }) {
  return (
    <div className="rounded-[10px] border border-hair px-2 py-1">
      <dt className={`text-[21px] leading-tight font-semibold tabular-nums ${tone ?? ""}`}>
        {many}
      </dt>
      <dd className="text-[9px] leading-tight text-faint">{said}</dd>
    </div>
  );
}
