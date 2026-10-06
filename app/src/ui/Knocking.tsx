import { ask, hush, shownAs, useKnocking } from "../knocking";
import { fill, t } from "../locales";
import Confirming from "./Confirming";

export default function Knocking() {
  const { waiting, quiet, asking } = useKnocking();
  if (waiting.length === 0 && !asking) return null;
  const one = waiting.length === 1 ? waiting[0] : null;

  return (
    <>
      {!quiet && waiting.length > 0 && (
        <div
          role="status"
          className="mx-2.5 mb-2 flex flex-col gap-1.5 rounded-[10px] border border-hue-amber/45 bg-sheet px-3 py-2.5"
        >
          <span className="flex items-center gap-2 text-[12.5px] font-semibold">
            <span aria-hidden="true" className="h-2 w-2 shrink-0 rounded-full bg-hue-amber" />
            {one ? fill("knockOne", shownAs(one)) : fill("knockMany", String(waiting.length))}
          </span>
          <span className="text-[11.5px] leading-relaxed text-soft">
            {t(one ? "knockWhy" : "knockManyWhy")}
          </span>
          <span className="mt-0.5 flex items-center gap-2">
            <button
              type="button"
              onClick={ask}
              className="rounded-[8px] bg-accent px-3 py-1 text-[12px] font-semibold text-bg"
            >
              {t("knockConfirm")}
            </button>
            <button
              type="button"
              onClick={hush}
              className="px-1.5 py-1 text-[12px] text-faint hover:text-ink"
            >
              {t("knockLater")}
            </button>
          </span>
        </div>
      )}
      {asking && <Confirming waiting={waiting} />}
    </>
  );
}

export function KnockDot() {
  const { waiting } = useKnocking();
  if (waiting.length === 0) return null;
  return (
    <span
      role="img"
      aria-label={t("knockDot")}
      className="absolute top-1 right-1 h-[7px] w-[7px] rounded-full bg-hue-amber"
    />
  );
}
