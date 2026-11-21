import type { Settings } from "../core";
import type { Run, Word } from "./Card";
import EarlierLimit from "./EarlierLimit";
import Signing, { type Signs } from "./Signing";
import { held, type Working } from "./working";

interface Props {
  kept: Settings | null;
  amend: (edit: (was: Settings) => Settings) => void;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
  signs: Signs;
}

export default function Writing({ kept, amend, busy, run, tell, signs }: Props) {
  return (
    <>
      <Signing busy={busy} run={run} tell={tell} signs={signs} />
      {kept && (
        <EarlierLimit
          kept={kept}
          amend={amend}
          held={held(busy, "earlier")}
          run={run}
          tell={tell}
        />
      )}
    </>
  );
}
