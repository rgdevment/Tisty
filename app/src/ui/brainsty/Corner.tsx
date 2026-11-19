import { useEffect, useState } from "react";
import type { Group, Network } from "../../brainsty";
import { fill, t } from "../../locales";
import Glyph from "../Glyph";
import Legend from "./Legend";

interface Props {
  readonly network: Network;
  readonly groups: readonly Group[];
  readonly lively: boolean;
  readonly paused: boolean;
  readonly onLively: (lively: boolean) => void;
  readonly onPaused: (paused: boolean) => void;
  readonly costShown: boolean;
  readonly cost: () => { ms: number; fps: number };
}

const QUIET_BUTTON =
  "flex items-center gap-1.5 rounded-[10px] border border-hair bg-bg/95 px-2.5 py-1.5 text-[11.5px] font-semibold shadow-lift hover:bg-hover hover:text-ink disabled:opacity-50 disabled:hover:bg-bg/95";

export default function Corner({
  network,
  groups,
  lively,
  paused,
  onLively,
  onPaused,
  costShown,
  cost,
}: Props) {
  return (
    <div className="pointer-events-none absolute bottom-9 left-3 z-[5] flex max-w-[calc(100%-24px)] items-end gap-1.5">
      <div className="pointer-events-auto">
        <Legend network={network} groups={groups} />
      </div>
      <button
        type="button"
        onClick={() => onLively(!lively)}
        aria-pressed={lively}
        title={t(lively ? "brainstyEffectsOff" : "brainstyEffectsOn")}
        className={`${QUIET_BUTTON} pointer-events-auto ${lively ? "text-accent" : "text-soft"}`}
      >
        <Glyph name="orbit" className="h-3 w-3" />
        {t("brainstyEffects")}
      </button>
      <button
        type="button"
        onClick={() => onPaused(!paused)}
        aria-pressed={paused}
        disabled={!lively}
        title={t(paused ? "brainstyResume" : "brainstyPause")}
        className={`${QUIET_BUTTON} pointer-events-auto text-soft`}
      >
        <Glyph name={paused ? "play" : "pause"} className="h-3 w-3" />
        {t(paused ? "brainstyResume" : "brainstyPause")}
      </button>
      {costShown && <Cost cost={cost} />}
    </div>
  );
}

function Cost({ cost }: { readonly cost: () => { ms: number; fps: number } }) {
  const [said, setSaid] = useState(cost);
  useEffect(() => {
    const every = window.setInterval(() => setSaid(cost()), 1000);
    return () => window.clearInterval(every);
  }, [cost]);
  return (
    <p className="rounded-[10px] border border-hair bg-bg/95 px-2.5 py-1.5 text-[11.5px] text-soft tabular-nums shadow-lift">
      {fill("brainstyCost", said.ms.toFixed(2), Math.round(said.fps).toString())}
    </p>
  );
}
