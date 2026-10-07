import { turnedOff } from "../apart";
import { type Carrying, revealed, type Settings } from "../core";
import { daysFrom, stamped } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import type { Run, Which, Word } from "./Card";
import Here from "./Here";
import { Band, mild, strong, Warned } from "./Rows";

const QUIET_DAYS = 3;

interface Props {
  state: Carrying;
  kept: Settings | null;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  fail: (word?: Word) => void;
  carry: (way?: "again") => void;
  pickFolder: () => void;
}

export default function Syncing({
  state,
  kept,
  busy,
  said,
  trouble,
  run,
  fail,
  carry,
  pickFolder,
}: Props) {
  const held = busy !== null;
  const carrying = busy === "sync";
  return (
    <>
      <Band label={t("syncing")} />
      <section className="rounded-[10px] border border-hair px-4 py-3.5">
        <p className="text-[12.5px] leading-relaxed text-soft">
          {state.chosen
            ? fill("syncOn", state.chosen)
            : state.sharedWas
              ? fill("syncOffRestored", state.sharedWas)
              : t("syncOff")}
        </p>
        {state.chosen && state.keeper && <Warned keeper={state.keeper} named={state.keptBy} />}
        <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
          {state.chosen ? (
            <>
              <button type="button" disabled={held} onClick={() => carry()} className={strong}>
                {carrying ? t("syncing_") : t("syncNow")}
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
            <button type="button" disabled={held} onClick={pickFolder} className={strong}>
              {t("turnSyncOn")}
            </button>
          )}
          <span className="ml-auto text-[11.5px] text-faint">
            {state.chosen
              ? fill("syncLast", state.last ? stamped(state.last) : t("syncNever"))
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
            <div className="mt-2.5 flex flex-wrap items-center gap-2.5 border-t border-hair pt-2.5">
              <span className="text-[11.5px] text-faint">{t("syncSetUp")}</span>
              <button type="button" disabled={held} onClick={pickFolder} className={mild}>
                {t("changeFolder")}
              </button>
              <button
                type="button"
                disabled={held}
                onClick={() => run("sync", turnedOff(kept), () => {})}
                className={mild}
              >
                {t("syncOffNow")}
              </button>
              <button
                type="button"
                disabled={held}
                title={t("syncAgainWhy")}
                onClick={() => carry("again")}
                className={mild}
              >
                {t("syncAgain")}
              </button>
            </div>
          </>
        )}
        {state.chosen && <Here held={held} />}
        {trouble?.card === "sync" && (
          <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>
        )}
        {said?.card === "sync" && <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>}
      </section>
    </>
  );
}
