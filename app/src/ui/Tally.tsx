import { useAsked } from "../asked";
import { allRoutines } from "../core";
import { fill, t } from "../locales";

interface Props {
  counts: Record<string, number>;
  onError?: (problem: unknown) => void;
}

export default function Tally({ counts, onError }: Props) {
  const closed = counts.archive ?? 0;
  const all = useAsked(() => allRoutines(), [closed, counts.routines], onError);

  if (!closed) return null;

  const told = all ?? [];
  const kept = told.reduce((sum, one) => sum + one.kept, 0);
  const owed = told.reduce((sum, one) => sum + one.owed, 0);
  const best = told.reduce((top, one) => Math.max(top, one.longest), 0);
  const missed = told.reduce((sum, one) => sum + (one.measurable ? one.skipped : 0), 0);

  const stories = counts.stories ?? 0;
  const traces = counts.traces ?? 0;
  const series = counts.routines ?? 0;
  const turns = Math.max(0, closed - stories - traces);

  return (
    <dl className="flex flex-wrap gap-1.5 px-2.5 pb-2">
      <Fact said={String(closed)} small={t("tallyClosed")} />
      <Fact said={String(stories)} small={t("layerStories")} />
      <Fact
        said={String(turns)}
        small={series === 1 ? t("tallyTurnsOne") : fill("tallyTurns", String(series))}
      />
      <Fact said={String(traces)} small={t("layerTrace")} />
      {owed > 0 && <Fact said={`${kept}/${owed}`} small={t("tallyKept")} />}
      {best > 0 && <Fact said={String(best)} small={t("tallyBest")} />}
      {missed > 0 && <Fact said={String(missed)} small={t("tallyMissed")} tone="text-urgent" />}
    </dl>
  );
}

function Fact({ said, small, tone }: { said: string; small: string; tone?: string }) {
  return (
    <div className="min-w-[68px] flex-1 rounded-[10px] border border-hair px-2 py-1">
      <dt className={`text-[13px] leading-tight font-semibold tabular-nums ${tone ?? ""}`}>
        {said}
      </dt>
      <dd className="text-[9px] leading-tight text-faint">{small}</dd>
    </div>
  );
}
