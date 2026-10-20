import { useEffect, useState } from "react";
import { turnedOff } from "../apart";
import { useAwaitsChoice } from "../coming";
import {
  type Carrying,
  moveDropped,
  movePending,
  revealed,
  type Settings,
  type Toward,
} from "../core";
import { daysFrom, stamped } from "../format";
import { cloudNamed, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { endMove, useMove } from "../relocation";
import { syncingSaid } from "../syncSaid";
import type { Run, Which, Word } from "./Card";
import Clouded from "./Clouded";
import { Band, Fold, mild, risky, strong, Warned } from "./Rows";
import { useSince } from "./useSince";

const QUIET_DAYS = 3;

interface Props {
  state: Carrying;
  kept: Settings | null;
  busy: Which | null;
  busySince?: number;
  said?: Word;
  trouble?: Word;
  run: Run;
  fail: (word?: Word) => void;
  carry: (way?: "again") => void;
  pickFolder: () => void;
  onMove: (resume?: Toward) => void;
}

function Pending({ state, onMove }: { state: Carrying; onMove: (resume?: Toward) => void }) {
  const [pending, setPending] = useState<Toward | null>(null);
  const running = useMove();

  useEffect(() => {
    if (!state) return;
    movePending()
      .then(setPending)
      .catch(() => setPending(null));
  }, [state]);

  if (!pending || running) return null;
  const where =
    pending.kind === "cloud"
      ? `${cloudNamed(pending.provider)} · ${pending.email}`
      : pending.kind === "folder"
        ? pending.at
        : t("welcomeAlone");
  return (
    <div
      role="status"
      className="mb-2.5 flex flex-wrap items-center gap-2.5 rounded-[10px] border border-hue-amber/40 px-3 py-2"
    >
      <span className="min-w-0 flex-1 text-[12.5px] text-ink">{fill("movePending", where)}</span>
      <button
        type="button"
        onClick={() => {
          endMove();
          void moveDropped()
            .then(() => setPending(null))
            .catch(() => undefined);
        }}
        className={mild}
      >
        {t("moveDrop")}
      </button>
      <button type="button" onClick={() => onMove(pending)} className={strong}>
        {t("moveAgain")}
      </button>
    </div>
  );
}

export default function Syncing({
  state,
  kept,
  busy,
  busySince,
  said,
  trouble,
  run,
  fail,
  carry,
  pickFolder,
  onMove,
}: Props) {
  const held = busy !== null;
  const carrying = busy === "sync";
  const running = useSince(carrying, busySince);
  const awaits = useAwaitsChoice();
  const told = said?.card === "sync" && !(awaits && said.busy);
  if (state.mode === "cloud")
    return (
      <>
        <Band label={t("whereItSyncs")} />
        <section className="rounded-[10px] border border-hair px-4 py-3.5">
          <Pending state={state} onMove={onMove} />
          <Clouded
            state={state}
            busy={busy}
            busySince={busySince}
            said={said}
            trouble={trouble}
            run={run}
            fail={fail}
            carry={carry}
            onMove={() => onMove()}
          />
        </section>
      </>
    );
  return (
    <>
      <Band label={t("whereItSyncs")} />
      <section className="rounded-[10px] border border-hair px-4 py-3.5">
        <Pending state={state} onMove={onMove} />
        <p className="text-[12.5px] leading-relaxed text-soft">
          {state.chosen
            ? fill("syncOn", state.chosen)
            : state.later
              ? t("syncLaterCard")
              : state.sharedWas
                ? fill("syncOffRestored", state.sharedWas)
                : state.sharedWasLater
                  ? t("syncOffRestoredLater")
                  : t("syncOff")}
        </p>
        {state.chosen && state.keeper && <Warned keeper={state.keeper} named={state.keptBy} />}
        <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
          {state.chosen ? (
            <>
              <button type="button" disabled={held} onClick={() => carry()} className={strong}>
                {carrying ? syncingSaid(running) : t("syncNow")}
              </button>
              <button
                type="button"
                disabled={held}
                onClick={() =>
                  state.chosen &&
                  revealed(state.chosen).catch((e) => fail({ card: "sync", text: saidPlainly(e) }))
                }
                className={mild}
              >
                {t("revealFolder")}
              </button>
            </>
          ) : (
            !state.later && (
              <button type="button" disabled={held} onClick={pickFolder} className={strong}>
                {t("turnSyncOn")}
              </button>
            )
          )}
          <span className="ml-auto text-[11.5px] text-faint">
            {state.chosen && awaits
              ? t("syncChoosing")
              : state.chosen
                ? fill("syncLast", state.last ? stamped(state.last) : t("syncNever"))
                : state.later
                  ? ""
                  : t("noDestination")}
          </span>
        </div>
        {state.chosen && (
          <>
            <p className="mt-2 text-[12.5px] text-soft">
              {state.heard ? fill("syncHeard", stamped(state.heard)) : t("syncHeardNever")}
            </p>
            {state.heard && -daysFrom(state.heard) >= QUIET_DAYS && (
              <p className="mt-1.5 text-[12.5px] leading-relaxed text-ink">
                {t("syncNothingSince")}
              </p>
            )}
            <p className="mt-1.5 text-[12.5px] leading-relaxed text-faint">{t("syncOnlyFolder")}</p>
            <div className="mt-3 border-t border-hair pt-3">
              <p className="mb-2 text-[11.5px] text-faint">{t("syncSetUp")}</p>
              <div className="grid grid-cols-2 gap-2">
                <button type="button" disabled={held} onClick={pickFolder} className={mild}>
                  {t("changeFolder")}
                </button>
                {state.cloudOffered && (
                  <button type="button" disabled={held} onClick={() => onMove()} className={mild}>
                    {t("moveOpen")}
                  </button>
                )}
              </div>
              <Fold>
                <button
                  type="button"
                  disabled={held}
                  title={t("syncAgainWhy")}
                  onClick={() => carry("again")}
                  className={mild}
                >
                  {t("syncAgain")}
                </button>
                <button
                  type="button"
                  disabled={held}
                  onClick={() => run("sync", turnedOff(kept), () => {})}
                  className={risky}
                >
                  {t("syncOffNow")}
                </button>
              </Fold>
            </div>
          </>
        )}
        {trouble?.card === "sync" && (
          <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>
        )}
        {told && said && <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>}
      </section>
    </>
  );
}
