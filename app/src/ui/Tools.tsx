import { checked, type Reviewed, rebuild } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import type { Run, Which, Word } from "./Card";
import { Line, mild } from "./Rows";

interface Props {
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
}

export const rebuildNow = (
  card: "index" | "rebuild",
  run: Run,
  setAudit: (audit: Reviewed) => void,
  tell: (word?: Word) => void,
) =>
  run(card, rebuild().then(checked), (now) => {
    setAudit(now);
    tell({ card, text: t("reviewRebuilt") });
  });

export default function Tools({
  audit,
  setAudit,
  busy,
  said,
  trouble,
  run,
  tell,
}: Readonly<Props>) {
  const held = busy !== null;

  return (
    <details className="mt-5">
      <summary className="cursor-pointer text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase hover:text-soft">
        {t("tools")}
      </summary>
      {audit && (
        <dl className="mt-2 grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-0.5 text-[12.5px]">
          <dt className="text-faint">{t("inTheLog")}</dt>
          <dd className="text-soft">
            {[
              fill("reviewCount", String(audit.tasks)),
              fill("reviewLists", String(audit.lists)),
              `${audit.events} ${t("wordEvents")}`,
            ].join(" · ")}
          </dd>
          <dt className="text-faint">{t("cacheIs")}</dt>
          <dd className={audit.agrees ? "text-accent" : "text-urgent"}>
            {t(audit.agrees ? "cacheAgrees" : "cacheDiverged")}
          </dd>
          <dt className="text-faint">{t("weighsLog")}</dt>
          <dd className="tabular-nums text-soft">{weigh(audit.logBytes)}</dd>
          <dt className="text-faint">{t("weighsDocs")}</dt>
          <dd className="tabular-nums text-soft">{weigh(audit.docsBytes)}</dd>
          <dt className="text-faint">{t("weighsHeld")}</dt>
          <dd className="tabular-nums text-soft">
            {`${audit.heldFiles} · ${weigh(audit.heldBytes)}`}
          </dd>
        </dl>
      )}
      <div className="mt-2 border-t border-hair">
        <Line
          title={t("quickIndex")}
          why={t("quickIndexWhy")}
          which="index"
          said={said}
          trouble={trouble}
        >
          <button
            type="button"
            disabled={held}
            onClick={() => rebuildNow("index", run, setAudit, tell)}
            className={mild}
          >
            {t("reviewRedo")}
          </button>
        </Line>
      </div>
    </details>
  );
}
