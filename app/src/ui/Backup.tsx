import type { ReactNode } from "react";
import type { Carrying } from "../core";
import { stamped, weigh } from "../format";
import { fill, t } from "../locales";
import type { Word } from "./Card";
import { Band, Line, mild, risky } from "./Rows";

interface Props {
  state: Carrying;
  holds: string;
  held: boolean;
  said?: Word;
  trouble?: Word;
  onMake: () => void;
  onTake: () => void;
  children?: ReactNode;
}

export default function Backup({
  state,
  holds,
  held,
  said,
  trouble,
  onMake,
  onTake,
  children,
}: Props) {
  return (
    <>
      <Band label={t("bandCopies")} />
      <div className="border-t border-hair">
        <Line
          title={t("backupSave")}
          why={
            <>
              <span className="block">
                {t(state.freesSpace ? "backupCloudWhat" : "backupWhat")}
              </span>
              <span className="mt-0.5 block tabular-nums">
                {[
                  holds,
                  state.weight > state.carries
                    ? fill("backupPastIt", weigh(state.weight), weigh(state.carries))
                    : fill("backupAbout", weigh(state.weight)),
                  state.backedUpAt ? stamped(state.backedUpAt) : t("backupNever"),
                ].join(" · ")}
              </span>
            </>
          }
          which="backup"
          said={said}
          trouble={trouble}
        >
          <button
            type="button"
            disabled={held || state.weight > state.carries}
            onClick={onMake}
            className={mild}
          >
            {t("backupMake")}
          </button>
        </Line>

        <Line
          title={t("restoreTitle")}
          why={t("restoreWhat")}
          which="restore"
          said={said}
          trouble={trouble}
        >
          <button type="button" disabled={held} onClick={onTake} className={risky}>
            {t("restoreFrom")}
          </button>
        </Line>
        {children}
      </div>
    </>
  );
}
