import { useEffect, useState } from "react";
import type { List, Series } from "../core";
import { allRoutines } from "../core";
import { cadence } from "../format";
import { fill, t } from "../locales";

interface Props {
  lists: List[];
  onOpen: (task: string) => void;
  onError?: (problem: unknown) => void;
}

export default function Shelf({ lists, onOpen, onError }: Props) {
  const [all, setAll] = useState<Series[] | null>(null);

  useEffect(() => {
    let alive = true;
    allRoutines()
      .then((some) => {
        if (alive) setAll(some);
      })
      .catch((problem) => onError?.(problem));
    return () => {
      alive = false;
    };
  }, [onError]);

  if (!all) return null;
  if (!all.length) {
    return <p className="px-2.5 py-4 text-sm leading-relaxed text-soft">{t("routinesEmpty")}</p>;
  }

  const named = (id?: string) => lists.find((one) => one.id === id)?.name;

  return (
    <div role="list" aria-label={t("layerRoutines")} className="flex flex-col gap-px">
      {all.map((one) => {
        const missing = one.measurable ? one.skipped : 0;
        return (
          <div
            key={one.last}
            role="listitem"
            tabIndex={0}
            onClick={() => onOpen(one.last)}
            onKeyDown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                onOpen(one.last);
              }
            }}
            className="grid cursor-pointer grid-cols-[3px_16px_minmax(0,1fr)_auto] items-start gap-2.5 rounded-lg py-2 pr-2.5 pl-1.5 outline-none hover:bg-hover focus-visible:ring-2 focus-visible:ring-accent"
          >
            <span aria-hidden="true" className="h-full min-h-5 rounded-sm bg-mark-list" />
            <span aria-hidden="true" className="pt-px text-center text-[13px] text-soft">
              ↻
            </span>
            <div className="min-w-0">
              <h2 className="text-sm leading-snug">{one.title}</h2>
              <div className="mt-0.5 flex flex-wrap gap-2.5 text-xs text-faint">
                {one.repeat && <span>{cadence(one.repeat)}</span>}
                {named(one.list) && <span className="text-soft">@{named(one.list)}</span>}
                {one.tags?.length ? (
                  <span>{one.tags.map((tag) => `#${tag}`).join(" ")}</span>
                ) : null}
                {one.streak > 0 && <span>{fill("shelfStreak", String(one.streak))}</span>}
              </div>
            </div>
            <div className="pt-px text-right text-xs whitespace-nowrap text-faint tabular-nums">
              <span>
                {one.kept}/{one.turns.length}
              </span>
              {missing > 0 && (
                <span className="ml-2 text-urgent">
                  {missing === 1 ? t("shelfMissedOne") : fill("shelfMissed", String(missing))}
                </span>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
