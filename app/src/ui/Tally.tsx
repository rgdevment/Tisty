import { useAsked } from "../asked";
import { allRoutines } from "../core";
import { t } from "../locales";

interface Props {
  counts: Record<string, number>;
  onError?: (problem: unknown) => void;
}

export default function Tally({ counts, onError }: Props) {
  const all = useAsked(() => allRoutines(), [], onError);
  const closed = counts.archive ?? 0;

  if (!closed) return null;

  const told = all ?? [];
  const kept = told.reduce((sum, one) => sum + one.kept, 0);
  const owed = told.reduce((sum, one) => sum + one.owed, 0);
  const best = told.reduce((top, one) => Math.max(top, one.longest), 0);
  const missed = told.reduce((sum, one) => sum + (one.measurable ? one.skipped : 0), 0);

  return (
    <dl className="flex flex-wrap gap-1.5 px-2.5 pb-2">
      <Fact said={String(closed)} small={t("tallyClosed")} />
      <Fact said={String(counts.stories ?? 0)} small={t("layerStories")} />
      <Fact said={String(counts.routines ?? 0)} small={t("layerRoutines")} />
      <Fact said={String(counts.traces ?? 0)} small={t("layerTrace")} />
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
