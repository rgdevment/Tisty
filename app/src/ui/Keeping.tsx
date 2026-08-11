import { useCallback, useEffect, useState } from "react";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import {
  backUp,
  checked,
  chooseSync,
  restore,
  syncNow,
  syncState,
  type Carrying,
  type Reviewed,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { stamped } from "../format";

interface Props {
  onChanged: () => void;
  onError: (problem: string) => void;
}

export default function Keeping({ onChanged, onError }: Props) {
  const [state, setState] = useState<Carrying | null>(null);
  const [audit, setAudit] = useState<Reviewed | null>(null);
  const [busy, setBusy] = useState(false);
  const [said, setSaid] = useState<string>();

  const look = useCallback(() => {
    syncState()
      .then(setState)
      .catch((e) => onError(saidPlainly(e)));
  }, [onError]);

  useEffect(look, [look]);

  const run = <T,>(work: Promise<T>, then: (answer: T) => void) => {
    setBusy(true);
    setSaid(undefined);
    work
      .then((answer) => {
        then(answer);
        look();
        onChanged();
      })
      .catch((e) => onError(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  if (!state) return <main className="overflow-hidden" />;

  const pickFolder = () => {
    open({ directory: true })
      .then((at) => typeof at === "string" && run(chooseSync(at), () => {}))
      .catch((e) => onError(saidPlainly(e)));
  };

  const makeBackup = () => {
    save({ defaultPath: "tisty-backup.zip", filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(
        (at) =>
          typeof at === "string" &&
          run(backUp(at), (bytes) => setSaid(fill("backupMade", weigh(bytes)))),
      )
      .catch((e) => onError(saidPlainly(e)));
  };

  const takeBackup = () => {
    open({ filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(async (at) => {
        if (typeof at !== "string") return;
        if (!(await ask(t("restoreSure"), { kind: "warning" }))) return;
        run(restore(at), (files) => setSaid(fill("restored", String(files))));
      })
      .catch((e) => onError(saidPlainly(e)));
  };

  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="scroller mx-auto w-full max-w-[560px] px-6 pb-12">
        <h2 className="mb-3.5 text-[21px] font-semibold">{t("keeping")}</h2>

        <Card title={t("syncing")}>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {state.chosen ? fill("syncOn", state.chosen) : t("syncOff")}
          </p>
          <div className="mt-2.5 flex items-center gap-2.5">
            {state.chosen ? (
              <>
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => run(syncNow(), (came) => setSaid(t(came ? "syncCame" : "syncSame")))}
                  className="rounded-[7px] bg-accent px-2.5 py-1 text-[12.5px] text-white"
                >
                  {busy ? t("syncing_") : t("syncNow")}
                </button>
                <button type="button" onClick={pickFolder} className={mild}>
                  {t("changeFolder")}
                </button>
                <button
                  type="button"
                  onClick={() => run(chooseSync(undefined), () => {})}
                  className={mild}
                >
                  {t("syncOffNow")}
                </button>
              </>
            ) : (
              <button
                type="button"
                onClick={pickFolder}
                className="rounded-[7px] bg-accent px-2.5 py-1 text-[12.5px] text-white"
              >
                {t("turnSyncOn")}
              </button>
            )}
            <span className="ml-auto text-[11.5px] text-faint">
              {state.chosen
                ? fill("syncLast", state.last ? stamped(state.last) : t("syncNever"))
                : t("noDestination")}
            </span>
          </div>
        </Card>

        <Card title={t("backup")}>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {state.backsUp ? t("backupWhat") : t("backupOffWhy")}
          </p>
          {state.backsUp && (
            <div className="mt-2.5 flex items-center gap-2.5">
              <button type="button" disabled={busy} onClick={makeBackup} className={mild}>
                {t("backupMake")}
              </button>
              <button type="button" disabled={busy} onClick={takeBackup} className={mild}>
                {t("backupRestore")}
              </button>
              {said && <span className="ml-auto text-[11.5px] text-faint">{said}</span>}
            </div>
          )}
        </Card>

        <Card title={t("review")}>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {audit
              ? [
                  fill("reviewCount", String(audit.tasks)),
                  fill("reviewLists", String(audit.lists)),
                  t(audit.agrees ? "reviewAgrees" : "reviewDiffers"),
                ].join(" · ")
              : fill("reviewLoose", String(state.loose))}
          </p>
          <div className="mt-2.5 flex items-center gap-2.5">
            <button
              type="button"
              disabled={busy}
              onClick={() => run(checked(), setAudit)}
              className={mild}
            >
              {t("reviewRun")}
            </button>
            {audit && audit.loose > 0 && (
              <span className="ml-auto text-[11.5px] text-faint">
                {`${fill("reviewLoose", String(audit.loose))} · ${weigh(audit.looseBytes)}`}
              </span>
            )}
          </div>
        </Card>
      </div>
    </main>
  );
}

const mild =
  "rounded-[7px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover disabled:opacity-50";

function Card({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="mb-3 rounded-[10px] border border-hair px-4 py-3.5">
      <h3 className="mb-0.5 text-[13.5px] font-semibold">{title}</h3>
      {children}
    </section>
  );
}

function weigh(bytes: number): string {
  const units = ["B", "kB", "MB", "GB"];
  let step = 0;
  let left = bytes;
  while (left >= 1000 && step < units.length - 1) {
    left /= 1000;
    step += 1;
  }
  return `${step === 0 ? left : left.toFixed(1)} ${units[step]}`;
}
