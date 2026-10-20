import { type Erasing, useErasing } from "../erasing";
import { filled, t } from "../locales";
import { dismissMove, type Run, useMove } from "../relocation";
import { alongSaid, doneSaid, pauseSaid, sayFor, useTicking, waitingSaid } from "./RelocatingSteps";

const said = (run: Run, erasing: Erasing | null): string => {
  if (run.failed) return t("moveStopped");
  if (erasing?.going)
    return erasing.erased?.of
      ? filled(t("moveErasingAlong"), String(erasing.erased.gone), String(erasing.erased.of))
      : t("moveErasingStill");
  if (run.going) return alongSaid(run) ?? t("moveMoving");
  const paused = pauseSaid(run);
  if (paused) return paused;
  if (run.result?.state === "waiting")
    return waitingSaid(run.result, run.failing, sayFor(run.toward));
  return run.result?.state === "done" ? doneSaid(run.toward) : t("moveWaiting");
};

export default function MoveNotice({ onOpen }: { onOpen: () => void }) {
  const run = useMove();
  const erasing = useErasing();
  useTicking(run?.pausedUntil !== undefined);
  if (!run || run.watched) return null;
  const done = run.result?.state === "done" && !run.going;
  // What becomes of the copies in a cloud left behind is asked only in the move's last step.
  const asksAboutCopies = run.result?.state === "done" && run.result.told && !!run.result.leftCloud;
  const failed = Boolean(run.failed);
  const waiting =
    !failed && !run.going && run.pausedUntil === undefined && run.result?.state === "waiting";
  const asks = failed || waiting;
  return (
    <div
      role={failed ? "alert" : "status"}
      aria-live={failed ? "assertive" : "polite"}
      className={`mx-2.5 mb-1 flex items-center gap-2 rounded-[10px] border px-2.5 py-1.5 ${
        asks ? "border-hue-amber/40" : "border-hair"
      }`}
    >
      <span
        className={`min-w-0 flex-1 text-[11.5px] ${
          failed ? "text-urgent" : done || waiting ? "text-ink" : "text-soft"
        }`}
      >
        {said(run, erasing)}
      </span>
      <button
        type="button"
        onClick={onOpen}
        className="shrink-0 text-[11.5px] text-accent hover:underline"
      >
        {t("moveShow")}
      </button>
      {done && (
        <button
          type="button"
          onClick={() => (asksAboutCopies ? onOpen() : dismissMove())}
          className="shrink-0 text-[11.5px] text-faint hover:text-soft"
        >
          {t("viewGotIt")}
        </button>
      )}
    </div>
  );
}
