import { type Stage, useComing } from "../coming";
import { fill, t, type Word } from "../locales";

const STAGES: Record<Stage, Word> = {
  log: "comingLog",
  papers: "comingPapers",
  attachments: "comingAttachments",
  uploading: "comingUploading",
};

/** A first round bringing a place's Tisty home: a looping bar, the counts as they come, and a way to go in. */
export default function Bringing({ from, onHide }: { from: string; onHide: () => void }) {
  const coming = useComing();
  const counted = coming && coming.whole > 0 ? coming : null;
  return (
    <div className="flex flex-col gap-3">
      <div
        role="status"
        aria-live="polite"
        className="rounded-[10px] border border-hair bg-accent-soft px-3 py-2 text-[11.5px] leading-relaxed text-soft"
      >
        <span className="block text-[12.5px] font-semibold text-ink">
          {fill("welcomeBringing", from)}
        </span>
        {t("welcomeCarryingWhy")}
        <span aria-hidden="true" className="mt-2 block h-1 overflow-hidden rounded-full bg-line">
          <span className="sliding block h-full w-1/3 rounded-full bg-accent" />
        </span>
        {counted && (
          <span className="mt-1.5 block tabular-nums text-faint">
            {`${t(STAGES[counted.stage])} ${fill("comingOf", String(counted.done), String(counted.whole))}`}
          </span>
        )}
      </div>
      <button
        type="button"
        onClick={onHide}
        className="self-end text-[11.5px] text-accent hover:underline"
      >
        {t("welcomeKeepWorking")}
      </button>
    </div>
  );
}
