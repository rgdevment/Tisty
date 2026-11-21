import { useEffect, useRef } from "react";
import {
  type About,
  type Carrying,
  checked,
  type Freeing,
  freeUp,
  type Holds,
  type Reviewed,
  revealed,
  type Settings,
  stopFreeing,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { settingAt } from "../settingsRows";
import type { Run, Which, Word } from "./Card";
import FreeUp from "./FreeUp";
import { Note, NoteButton } from "./RowNotes";
import { Band, Line, mild, off } from "./Rows";
import { held as isHeld, type Working } from "./working";

interface Props {
  state: Carrying;
  kept: Settings | null;
  build: About | null;
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  freeing: Freeing | null;
  setFreeing: (freeing: Freeing | null) => void;
  freeDays: number;
  setFreeDays: (days: number) => void;
  busy: Working;
  run: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  clear: (card: Which) => void;
  carry: () => void;
  remember: (next: Settings, card: Which) => void;
  onDoc?: (paper: string) => void;
  onSync?: () => void;
}

export default function Storage(props: Props) {
  const { state, kept, build, audit, setAudit, busy, fail } = props;
  const held = busy.size > 0;

  // Read once a visit, after any card still working has landed, and not when the settings already hold it.
  const asked = useRef(audit !== null);
  useEffect(() => {
    if (held || asked.current) return;
    asked.current = true;
    checked()
      .then(setAudit)
      .catch((e) => fail({ card: "store", text: saidPlainly(e) }));
  }, [held, setAudit, fail]);

  return (
    <>
      <Band label={t("whereItLives")} />
      <div className="border-t border-hair">
        <Line
          row="folder"
          why={
            <>
              <span className="block font-mono text-[11.5px] break-all">{build?.store ?? "…"}</span>
              <span className="mt-0.5 block">{t("storeFixed")}</span>
            </>
          }
          which="store"
          quiet
        >
          <button
            type="button"
            disabled={!build}
            onClick={() =>
              build &&
              revealed(build.store)
                .then(() => props.clear("store"))
                .catch((e) => fail({ card: "store", text: saidPlainly(e) }))
            }
            className={mild}
          >
            {t("aboutReveal")}
          </button>
        </Line>
      </div>

      {audit && <Weights audit={audit} />}

      {!state.freesSpace && kept && <HoldsLine {...props} kept={kept} />}

      <FreeUp
        audit={audit}
        setAudit={setAudit}
        build={build}
        state={state}
        freeing={props.freeing}
        setFreeing={props.setFreeing}
        days={props.freeDays}
        setDays={props.setFreeDays}
        onDoc={props.onDoc}
        busy={busy}
        run={props.run}
        tell={props.tell}
        fail={fail}
        clear={props.clear}
      />
    </>
  );
}

function Weights({ audit }: { audit: Reviewed }) {
  return (
    <div {...settingAt("weighs")}>
      <Band label={t("storageWeighs")} />
      <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-0.5 border-t border-hair pt-2 text-[12.5px]">
        <dt className="text-faint">{t("inTheLog")}</dt>
        <dd className="text-soft">
          {[
            fill("reviewCount", String(audit.tasks)),
            fill("reviewLists", String(audit.lists)),
            `${audit.events} ${t("wordEvents")}`,
          ].join(" · ")}
        </dd>
        <dt className="text-faint">{t("weighsLog")}</dt>
        <dd className="tabular-nums text-soft">{weigh(audit.logBytes)}</dd>
        <dt className="text-faint">{t("weighsDocs")}</dt>
        <dd className="tabular-nums text-soft">{weigh(audit.docsBytes)}</dd>
        <dt className="text-faint">{t("weighsHeld")}</dt>
        <dd className="tabular-nums text-soft">{`${audit.heldFiles} · ${weigh(audit.heldBytes)}`}</dd>
      </dl>
    </div>
  );
}

function HoldsLine({
  kept,
  freeing,
  setFreeing,
  busy,
  fail,
  carry,
  remember,
  onSync,
}: Props & { kept: Settings }) {
  return (
    <>
      <Band label={t("attachTitle")} />
      <div className="border-t border-hair">
        <Line
          row="holds"
          why={kept.shares ? fill("holdsWhy", weigh(kept.onlySharedAbove)) : undefined}
          which="holds"
          note={
            !kept.shares && (
              <Note
                kind="dep"
                act={onSync && <NoteButton label={t("goToSync")} onPress={onSync} />}
              >
                {t("holdsNeedsShared")}
              </Note>
            )
          }
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
            disabled={isHeld(busy, "holds") || !kept.shares}
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
  );
}
