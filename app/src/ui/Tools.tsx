import { checked, type Reviewed, rebuild } from "../core";
import { t } from "../locales";
import type { Run, Word } from "./Card";
import { Band, Line, mild } from "./Rows";
import { held, type Working } from "./working";

interface Props {
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
}

const rebuildNow = (run: Run, setAudit: (audit: Reviewed) => void, tell: (word?: Word) => void) =>
  run(
    "index",
    () => rebuild().then(checked),
    (now) => {
      setAudit(now);
      tell({ card: "index", text: t("reviewRebuilt") });
    },
  );

export default function Tools({ audit, setAudit, busy, run, tell }: Readonly<Props>) {
  return (
    <>
      <Band label={t("tools")} />
      <div className="border-t border-hair">
        <Line row="index" why={t("quickIndexWhy")} which="index">
          {audit && (
            <span className={`text-[12.5px] ${audit.agrees ? "text-accent" : "text-urgent"}`}>
              {t(audit.agrees ? "cacheAgrees" : "cacheDiverged")}
            </span>
          )}
          <button
            type="button"
            disabled={held(busy, "index")}
            onClick={() => rebuildNow(run, setAudit, tell)}
            className={mild}
          >
            {t("reviewRedo")}
          </button>
        </Line>
      </div>
    </>
  );
}
