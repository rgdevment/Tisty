import {
  type Carrying,
  type Freeing,
  freeUp,
  type Holds,
  type Reviewed,
  type Settings,
  stopFreeing,
  type Toward,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import type { Run, Which, Word } from "./Card";
import Computers from "./Computers";
import { Band, Line, off } from "./Rows";
import Syncing from "./Syncing";

interface Props {
  state: Carrying;
  kept: Settings | null;
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  freeing: Freeing | null;
  setFreeing: (freeing: Freeing | null) => void;
  busy: Which | null;
  busySince?: number;
  said?: Word;
  trouble?: Word;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  carry: (way?: "again") => void;
  pickFolder: () => void;
  onMove: (resume?: Toward) => void;
  remember: (next: Settings, card: Which) => void;
}

export default function Sync({
  state,
  kept,
  audit,
  setAudit,
  freeing,
  setFreeing,
  busy,
  busySince,
  said,
  trouble,
  run,
  tell,
  fail,
  carry,
  pickFolder,
  onMove,
  remember,
}: Props) {
  const held = busy !== null;
  return (
    <>
      <Syncing
        state={state}
        kept={kept}
        busy={busy}
        busySince={busySince}
        said={said}
        trouble={trouble}
        run={run}
        fail={fail}
        carry={carry}
        pickFolder={pickFolder}
        onMove={onMove}
      />

      <Computers
        state={state}
        audit={audit}
        setAudit={setAudit}
        busy={busy}
        said={said}
        trouble={trouble}
        run={run}
        tell={tell}
        fail={fail}
      />

      {!state.freesSpace && kept && (
        <>
          <Band label={t("attachTitle")} />
          <div className="border-t border-hair">
            <Line
              title={t("holdsTitle")}
              why={
                kept.shares ? fill("holdsWhy", weigh(kept.onlySharedAbove)) : t("holdsNeedsShared")
              }
              which="holds"
              said={said}
              trouble={trouble}
              more={
                freeing && (
                  <div className="mt-2 flex items-center gap-2.5">
                    <span className="text-[11.5px] leading-relaxed text-soft">
                      {fill(freeing.done ? "holdsFreed" : "holdsFreeing", weigh(freeing.freed))}
                    </span>
                    {!freeing.done && (
                      <button
                        type="button"
                        onClick={() => void stopFreeing()}
                        className="rounded-md border border-line px-2.5 py-0.5 text-[11.5px] text-soft hover:border-urgent hover:text-urgent"
                      >
                        {t("holdsStop")}
                      </button>
                    )}
                  </div>
                )
              }
            >
              <select
                aria-label={t("holdsTitle")}
                value={kept.holds}
                disabled={held || !kept.shares}
                onChange={(e) => {
                  const holds = e.target.value as Holds;
                  remember({ ...kept, holds }, "holds");
                  if (holds === "shared") {
                    setFreeing({ gone: 0, freed: 0, done: false });
                    freeUp().catch((e) => {
                      setFreeing(null);
                      fail({ card: "holds", text: saidPlainly(e) });
                    });
                  }
                  if (holds === "everywhere") {
                    carry();
                  }
                }}
                className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
              >
                <option value="everywhere">{t("holdsEverywhere")}</option>
                <option value="mine">{t("holdsMine")}</option>
                <option value="shared">{t("holdsShared")}</option>
              </select>
            </Line>
          </div>
        </>
      )}
    </>
  );
}
