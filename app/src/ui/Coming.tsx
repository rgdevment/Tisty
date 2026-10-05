import { useEffect, useState } from "react";
import { type Stage, useComing } from "../coming";
import { fill, locale, t, type Word } from "../locales";

export const SHOWN_AFTER = 3_000;
export const SLOW_AFTER = 60_000;

const said: Record<Stage, Word> = {
  log: "comingLog",
  papers: "comingPapers",
  attachments: "comingAttachments",
};

const counted = (many: number): string => new Intl.NumberFormat(locale()).format(many);

export default function Coming() {
  const coming = useComing();
  const [, tick] = useState(0);

  useEffect(() => {
    if (!coming) return;
    const ticking = setInterval(() => tick((was) => was + 1), 1_000);
    return () => clearInterval(ticking);
  }, [coming]);

  if (!coming) return null;
  const long = Date.now() - coming.since;
  if (long < SHOWN_AFTER) return null;
  const far = coming.whole ? Math.min(100, Math.round((coming.done / coming.whole) * 100)) : 0;

  return (
    <div role="status" aria-live="polite" className="mx-2.5 mb-2 flex flex-col gap-1.5 px-2.5 py-2">
      <span className="flex items-baseline gap-1.5 text-[11.5px]">
        <span className="flex-1 truncate text-ink">{t(said[coming.stage])}</span>
        {coming.whole > 0 && (
          <span className="text-soft tabular-nums">
            {fill("comingOf", counted(coming.done), counted(coming.whole))}
          </span>
        )}
      </span>
      <span aria-hidden="true" className="block h-[3px] overflow-hidden rounded-full bg-line">
        <span
          className="block h-full rounded-full bg-accent motion-safe:transition-[width]"
          style={{ width: `${far}%` }}
        />
      </span>
      {long >= SLOW_AFTER && <span className="text-[11.5px] text-soft">{t("comingSlow")}</span>}
    </div>
  );
}
