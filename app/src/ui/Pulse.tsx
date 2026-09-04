import type { Counted, List } from "../core";
import { fill, t } from "../locales";
import { QUADRANTS, said, tint } from "../quadrants";

interface Props {
  counts: Record<string, number>;
  lists: List[];
  tags: Counted[];
  papers: number;
  onList: (id: string) => void;
  onTags: () => void;
  onQuadrants: () => void;
}

const SHOWN = 8;

export default function Pulse({ counts, lists, tags, papers, onList, onTags, onQuadrants }: Props) {
  const held = lists
    .filter((one) => counts[one.id])
    .sort((a, b) => (counts[b.id] ?? 0) - (counts[a.id] ?? 0));
  const named = [...tags].sort((a, b) => b.tasks - a.tasks).slice(0, SHOWN);

  return (
    <aside className="scroller flex flex-col gap-4 border-l border-hair bg-panel px-3.5 pt-12 pb-6">
      <div>
        <Cap said={t("theDay")} />
        <dl className="grid grid-cols-3 gap-1.5">
          <Count many={counts.overdue ?? 0} said={t("pulseLate")} tone="text-urgent" />
          <Count many={counts.dueToday ?? 0} said={t("pulseToday")} tone="text-accent" />
          <Count many={counts.upcoming ?? 0} said={t("pulseAhead")} />
        </dl>
      </div>

      <div>
        <Cap said={t("quadrants")} />
        <div className="grid grid-cols-2 gap-1.5">
          {QUADRANTS.map((one) => (
            <button
              key={one}
              type="button"
              onClick={onQuadrants}
              className="flex items-baseline gap-2 rounded-lg border border-hair px-2 py-1 text-left hover:bg-hover"
            >
              <span className="min-w-0 truncate text-[11px] text-soft">{said(one)}</span>
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
            className="mt-1 flex w-full items-baseline gap-2 text-left text-[11px] text-faint hover:text-ink"
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

      {named.length > 0 && (
        <div>
          <Cap said={t("tags")} />
          <div className="flex flex-wrap gap-1">
            {named.map((one) => (
              <button
                key={one.tag}
                type="button"
                onClick={onTags}
                className="rounded-full border border-hair px-2 text-[10.5px] text-faint hover:text-ink"
              >
                {fill("tagAndCount", one.tag, String(one.tasks))}
              </button>
            ))}
          </div>
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
    <div className="rounded-lg border border-hair px-2 py-1">
      <dt className={`text-[17px] leading-tight font-semibold tabular-nums ${tone ?? ""}`}>
        {many}
      </dt>
      <dd className="text-[9.5px] leading-tight text-faint">{said}</dd>
    </div>
  );
}
