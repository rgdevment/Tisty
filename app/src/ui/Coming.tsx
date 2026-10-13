import { useEffect, useState } from "react";
import { type Stage, useComing, useStuck } from "../coming";
import { fill, locale, t, type Word } from "../locales";
import { leftBehind, plainly } from "../refusal";
import type { Heading } from "../views";

export const SHOWN_AFTER = 3_000;
export const SLOW_AFTER = 60_000;

const said: Record<Stage, Word> = {
  log: "comingLog",
  papers: "comingPapers",
  attachments: "comingAttachments",
};

const counted = (many: number): string => new Intl.NumberFormat(locale()).format(many);

const followed = (stuck: unknown): { heading: Heading; said: Word } => {
  const code = (stuck as { code?: string } | null)?.code;
  if (code === "movedToCloud") return { heading: "cloud", said: "movedFollowCloud" };
  if (code === "movedAway") return { heading: "folder", said: "movedFollowFolder" };
  return { heading: "pick", said: "movedChoose" };
};

export default function Coming({ onOpen }: { onOpen?: (heading: Heading) => void }) {
  const coming = useComing();
  const stuck = useStuck();
  const [, tick] = useState(0);

  useEffect(() => {
    if (!coming) return;
    const ticking = setInterval(() => tick((was) => was + 1), 1_000);
    return () => clearInterval(ticking);
  }, [coming]);

  const long = coming ? Date.now() - coming.since : 0;
  const shown = coming !== null && long >= SHOWN_AFTER;
  if (!shown && !stuck) return null;
  const far = coming?.whole ? Math.min(100, Math.round((coming.done / coming.whole) * 100)) : 0;

  return (
    <div className="mx-2.5 mb-2 flex flex-col gap-1.5 px-2.5 py-2">
      {shown && coming && (
        <div role="status" aria-live="polite" className="flex flex-col gap-1.5">
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
      )}
      {stuck != null && (
        <p role="alert" className="text-[11.5px] text-urgent">
          {plainly(stuck)}
        </p>
      )}
      {stuck != null && onOpen && leftBehind(stuck) && (
        <button
          type="button"
          onClick={() => onOpen(followed(stuck).heading)}
          className="self-start rounded-[10px] border border-line px-2.5 py-1 text-[11.5px] hover:bg-hover"
        >
          {t(followed(stuck).said)}
        </button>
      )}
    </div>
  );
}
