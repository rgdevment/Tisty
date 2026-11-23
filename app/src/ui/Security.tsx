import { useCallback, useState } from "react";
import { unstuck } from "../coming";
import type { Carrying, Reviewed } from "../core";
import type { Identity as Shown } from "../identity";
import type { Run, Which, Word } from "./Card";
import Computers from "./Computers";
import Identity from "./Identity";
import Sealing from "./Sealing";
import { useCarryOn } from "./useCarryOn";
import type { Working } from "./working";

interface Props {
  state: Carrying;
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onChanged: () => void;
  onIdentity?: () => void;
  onSync?: () => void;
  carry?: () => void;
  clear?: (card: Which) => void;
}

export default function Security({
  state,
  audit,
  setAudit,
  busy,
  run,
  tell,
  fail,
  onChanged,
  onIdentity,
  onSync,
  carry,
  clear,
}: Props) {
  const [phrased, setPhrased] = useState(0);
  const phrasedAgain = useCallback(() => setPhrased((was) => was + 1), []);
  const [ours, setOurs] = useState<ReadonlySet<string>>(new Set());
  const known = useCallback(
    (shown: Shown) => {
      setOurs(new Set(shown.machines.filter((one) => one.ours).map((one) => one.id)));
      onIdentity?.();
    },
    [onIdentity],
  );
  const [locked, setLocked] = useState(false);
  const carryOn = useCarryOn(() => carry?.());
  const typed = () => {
    if (state.mode !== "cloud") return;
    unstuck();
    carryOn();
  };
  const cards = { busy, tell, fail, clear };
  return (
    <>
      <Identity
        round={state.last}
        phrased={phrased}
        onPhrased={phrasedAgain}
        onTyped={typed}
        opensCloud={locked}
        onShown={known}
        {...cards}
      />
      <Computers state={state} audit={audit} setAudit={setAudit} run={run} ours={ours} {...cards} />
      <div className="mt-4">
        <Sealing
          round={state.last}
          phrased={phrased}
          onPhrased={phrasedAgain}
          onChanged={onChanged}
          onSync={onSync}
          onLocked={setLocked}
          carry={carryOn}
          run={run}
          {...cards}
        />
      </div>
    </>
  );
}
