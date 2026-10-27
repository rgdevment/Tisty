import { useEffect, useState } from "react";
import { apartWaits, type Stage, useAside, useCarded, useComing, useStuck } from "../coming";
import { numbered } from "../format";
import { fill, t, type Word } from "../locales";
import { leftBehind, plainly } from "../refusal";
import { underway, useMove } from "../relocation";
import { useSyncSeen } from "../syncSeen";
import type { Heading } from "../views";

export const SHOWN_AFTER = 3_000;
export const SLOW_AFTER = 60_000;

const said: Record<Stage, Word> = {
  log: "comingLog",
  papers: "comingPapers",
  attachments: "comingAttachments",
  uploading: "comingUploading",
};

const followed = (stuck: unknown): { heading: Heading; said: Word } => {
  const code = (stuck as { code?: string } | null)?.code;
  if (code === "movedToCloud") return { heading: "cloud", said: "movedFollowCloud" };
  if (code === "movedAway") return { heading: "folder", said: "movedFollowFolder" };
  return { heading: "pick", said: "movedChoose" };
};

export default function Coming({ onOpen }: { onOpen?: (heading: Heading) => void }) {
  const coming = useComing();
  const told = useStuck();
  const aside = useAside();
  const carded = useCarded();
  const reading = useSyncSeen().status?.reading ?? null;
  // A read under way makes the last round's trouble old news.
  const stuck =
    reading || ((told as { code?: string } | null)?.code === carded && carded) ? null : told;
  const move = useMove();
  const [, tick] = useState(0);

  useEffect(() => {
    if (!coming) return;
    const ticking = setInterval(() => tick((was) => was + 1), 1_000);
    return () => clearInterval(ticking);
  }, [coming]);

  const long = coming ? Date.now() - coming.since : 0;
  // A move tells its own steps beside this, so the round's raw stages would only say them twice.
  const moving = move !== null && underway(move);
  const step =
    coming ?? (reading && { stage: "log" as Stage, done: reading.done, whole: reading.whole });
  const shown = ((coming !== null && long >= SHOWN_AFTER) || (!coming && reading)) && !moving;
  if (!shown && !stuck && !aside) return null;
  const far = step?.whole ? Math.min(100, Math.round((step.done / step.whole) * 100)) : 0;

  return (
    <div className="mx-2.5 mb-2 flex flex-col gap-1.5 px-2.5 py-2">
      {shown && step && (
        <div role="status" aria-live="polite" className="flex flex-col gap-1.5">
          <span className="flex items-baseline gap-1.5 text-[11.5px]">
            <span className="flex-1 truncate text-ink">{t(said[step.stage])}</span>
            {step.whole > 0 && (
              <span className="text-soft tabular-nums">
                {fill("comingOf", numbered(step.done), numbered(step.whole))}
              </span>
            )}
          </span>
          <span aria-hidden="true" className="block h-[3px] overflow-hidden rounded-full bg-line">
            <span
              className="block h-full rounded-full bg-accent motion-safe:transition-[width]"
              style={{ width: `${far}%` }}
            />
          </span>
          {coming && long >= SLOW_AFTER && (
            <span className="text-[11.5px] text-soft">
              {t(coming.stage === "uploading" ? "comingSlowUp" : "comingSlow")}
            </span>
          )}
        </div>
      )}
      {aside && stuck == null && (
        <p role="alert" className="text-[11.5px] text-soft">
          {aside}
        </p>
      )}
      {stuck != null && (
        <p role="alert" className="text-[11.5px] text-urgent">
          {apartWaits(stuck) ? t("stuckChoosing") : plainly(stuck)}
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
