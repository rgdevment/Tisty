import { useEffect, useState } from "react";
import type { Group, Network } from "../../brainsty";
import { fill } from "../../locales";
import Legend, { type Motions } from "./Legend";

interface Props {
  readonly network: Network;
  readonly groups: readonly Group[];
  readonly motions: Motions;
  readonly onMotions: (motions: Motions) => void;
  readonly costShown: boolean;
  readonly cost: () => { ms: number; fps: number };
}

export default function Corner({ network, groups, motions, onMotions, costShown, cost }: Props) {
  return (
    <div className="pointer-events-none absolute bottom-9 left-3 z-[5] flex max-w-[calc(100%-24px)] items-end gap-1.5">
      <div className="pointer-events-auto min-w-0">
        <Legend network={network} groups={groups} motions={motions} onMotions={onMotions} />
      </div>
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
