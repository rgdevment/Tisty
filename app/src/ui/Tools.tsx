import { useState } from "react";
import { checked, type Reviewed, readTags, rebuild } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { type Brittle, scanned } from "../scanning";
import { type Run, type Which, type Word, working } from "./Card";
import { Line, mild } from "./Rows";

interface Props {
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  busy: Which | null;
  secs: number;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
}

export default function Tools({ audit, setAudit, busy, secs, said, trouble, run, tell }: Props) {
  const held = busy !== null;
  const [brittle, setBrittle] = useState<Brittle[] | null>(null);

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
          title={t("tagsRead")}
          why={t("tagsReadWhy")}
          which="tagging"
          said={said}
          trouble={trouble}
        >
          <button
            type="button"
            disabled={held}
            onClick={() =>
              run("tagging", readTags(), (many) =>
                tell({ card: "tagging", text: fill("tagsReadDone", String(many)) }),
              )
            }
            className={mild}
          >
            {busy === "tagging" ? working("tagsReading", secs) : t("tagsReadDo")}
          </button>
        </Line>
        <Line
          title={t("brittleAre")}
          why={t("brittleWhat")}
          which="brittle"
          said={said}
          trouble={trouble}
          more={
            brittle && (
              <>
                {brittle.length === 0 && (
                  <p className="mt-2 text-[12.5px] text-faint">{t("brittleNone")}</p>
                )}
                {brittle.length > 0 && (
                  <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1.5 overflow-y-auto text-[12.5px]">
                    {brittle.map((one) => (
                      <li key={one.file}>
                        <span className="text-soft">{one.title || one.file}</span>
                        <span className="block text-[11.5px] text-faint">
                          {one.brings.map((what) => t(what as Parameters<typeof t>[0])).join(" · ")}
                        </span>
                      </li>
                    ))}
                  </ul>
                )}
              </>
            )
          }
        >
          <button
            type="button"
            disabled={held}
            onClick={() => run("brittle", scanned(), setBrittle)}
            className={mild}
          >
            {busy === "brittle"
              ? working("brittleReading", secs)
              : t(brittle ? "brittleAgain" : "brittleRun")}
          </button>
        </Line>
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
            onClick={() =>
              run("index", rebuild().then(checked), (now) => {
                setAudit(now);
                tell({ card: "index", text: t("reviewRebuilt") });
              })
            }
            className={mild}
          >
            {t("reviewRedo")}
          </button>
        </Line>
      </div>
    </details>
  );
}
