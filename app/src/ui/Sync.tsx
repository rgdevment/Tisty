import type { Carrying, Settings, Toward } from "../core";
import { synced } from "../footing";
import { t } from "../locales";
import type { Run, Word } from "./Card";
import { Line, mild } from "./Rows";
import Syncing from "./Syncing";
import type { Working } from "./working";

interface Props {
  state: Carrying;
  kept: Settings | null;
  busy: Working;
  run: Run;
  fail: (word?: Word) => void;
  carry: (way?: "again") => void;
  pickFolder: () => void;
  onMove: (resume?: Toward) => void;
  onSecurity: () => void;
}

export default function Sync({ onSecurity, ...syncing }: Props) {
  return (
    <Syncing {...syncing}>
      {synced(syncing.state) && (
        <div className="mt-5 border-t border-hair">
          <Line row="syncMachines" which="machines" quiet>
            <button type="button" onClick={onSecurity} className={mild}>
              {t("seeInSecurity")}
            </button>
          </Line>
        </div>
      )}
    </Syncing>
  );
}
