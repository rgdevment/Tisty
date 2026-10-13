import { stamped } from "../format";
import { fill, t } from "../locales";
import { type Run, useMove } from "../relocation";
import { doneSaid } from "./RelocatingSteps";

const said = (run: Run): string => {
  if (run.failed) return t("moveStopped");
  if (run.going) return t("moveMoving");
  if (run.pausedUntil !== undefined)
    return fill("movePaused", stamped(new Date(run.pausedUntil).toISOString()));
  return run.result?.state === "done" ? doneSaid(run.toward) : t("moveWaiting");
};

export default function MoveNotice({ onOpen }: { onOpen: () => void }) {
  const run = useMove();
  if (!run || run.watched) return null;
  const done = run.result?.state === "done" && !run.going;
  return (
    <div
      role="status"
      aria-live="polite"
      className="mx-2.5 mb-1 flex items-center gap-2 rounded-[10px] border border-hair px-2.5 py-1.5"
    >
      <span className={`min-w-0 flex-1 text-[11.5px] ${done ? "text-ink" : "text-soft"}`}>
        {said(run)}
      </span>
      <button
        type="button"
        onClick={onOpen}
        className="shrink-0 text-[11.5px] text-accent hover:underline"
      >
        {t("moveShow")}
      </button>
    </div>
  );
}
