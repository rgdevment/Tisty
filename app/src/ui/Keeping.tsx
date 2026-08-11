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

type Which = "sync" | "backup" | "review";
type Word = { card: Which; text: string };

interface Props {
  onChanged: () => void;
}

export default function Keeping({ onChanged }: Props) {
  const [state, setState] = useState<Carrying | null>(null);
  const [audit, setAudit] = useState<Reviewed | null>(null);
  const [busy, setBusy] = useState<Which | null>(null);
  const [said, setSaid] = useState<Word>();
  // Kept in the card, not in the window's banner: an unreachable folder is news
  // for the panel you came to read, and it would otherwise hang over every view.
  const [trouble, setTrouble] = useState<Word>();

  const look = useCallback(() => {
    syncState()
      .then(setState)
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
  }, []);

  useEffect(look, [look]);

  const run = <T,>(card: Which, work: Promise<T>, then: (answer: T) => void) => {
    setBusy(card);
    setSaid(undefined);
    setTrouble(undefined);
    work
      .then((answer) => {
        then(answer);
        look();
        onChanged();
      })
      .catch((e) => setTrouble({ card, text: saidPlainly(e) }))
      .finally(() => setBusy(null));
  };

  if (!state) return <main className="overflow-hidden" />;

  const carrying = busy === "sync";

  // Joining two histories cannot be undone, so the answer is the person's.
  const carryNow = () => {
    if (carrying) return;
    setBusy("sync");
    setSaid(undefined);
    setTrouble(undefined);
    syncNow()
      .then((came) => setSaid({ card: "sync", text: t(came ? "syncCame" : "syncSame") }))
      .catch(async (problem) => {
        const refusal = problem as { code?: string; name?: string };
        if (refusal?.code !== "wouldMerge") throw problem;
        if (!(await ask(fill("joinThem", refusal.name ?? ""), { kind: "warning" }))) return;
        const came = await syncNow(undefined, true);
        setSaid({ card: "sync", text: t(came ? "syncCame" : "syncSame") });
      })
      .then(() => {
        look();
        onChanged();
      })
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }))
      .finally(() => setBusy(null));
  };

  const pickFolder = () => {
    if (carrying) return;
    open({ directory: true })
      .then((at) => typeof at === "string" && run("sync", chooseSync(at), () => {}))
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
  };

  const makeBackup = () => {
    if (busy) return;
    save({ defaultPath: "tisty-backup.zip", filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(
        (at) =>
          typeof at === "string" &&
          run("backup", backUp(at), (bytes) =>
            setSaid({ card: "backup", text: fill("backupMade", weigh(bytes)) }),
          ),
      )
      .catch((e) => setTrouble({ card: "backup", text: saidPlainly(e) }));
  };

  const takeBackup = () => {
    if (busy) return;
    open({ filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(async (at) => {
        if (typeof at !== "string") return;
        if (!(await ask(t("restoreSure"), { kind: "warning" }))) return;
        run("backup", restore(at), (files) =>
          setSaid({ card: "backup", text: fill("restored", String(files)) }),
        );
      })
      .catch((e) => setTrouble({ card: "backup", text: saidPlainly(e) }));
  };

  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="scroller mx-auto w-full max-w-[560px] px-6 pb-12">
        <h2 className="mb-3.5 text-[21px] font-semibold">{t("keeping")}</h2>

        <Card title={t("syncing")} which="sync" said={said} trouble={trouble}>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {state.chosen ? fill("syncOn", state.chosen) : t("syncOff")}
          </p>
          <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
            {state.chosen ? (
              <>
                <button
                  type="button"
                  disabled={carrying}
                  onClick={carryNow}
                  className={strong}
                >
                  {carrying ? t("syncing_") : t("syncNow")}
                </button>
                <button type="button" disabled={carrying} onClick={pickFolder} className={mild}>
                  {t("changeFolder")}
                </button>
                <button
                  type="button"
                  disabled={carrying}
                  onClick={() => run("sync", chooseSync(undefined), () => {})}
                  className={mild}
                >
                  {t("syncOffNow")}
                </button>
              </>
            ) : (
              <button type="button" disabled={carrying} onClick={pickFolder} className={strong}>
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

        <Card title={t("backup")} which="backup" said={said} trouble={trouble}>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {state.backsUp ? t("backupWhat") : t("backupOffWhy")}
          </p>
          {state.backsUp && (
            <div className="mt-2.5 flex items-center gap-2.5">
              <button type="button" disabled={busy !== null} onClick={makeBackup} className={mild}>
                {t("backupMake")}
              </button>
              <button type="button" disabled={busy !== null} onClick={takeBackup} className={mild}>
                {t("backupRestore")}
              </button>
            </div>
          )}
        </Card>

        <Card title={t("review")} which="review" said={said} trouble={trouble}>
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
              disabled={busy === "review"}
              onClick={() => run("review", checked(), setAudit)}
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
const strong = "rounded-[7px] bg-accent px-2.5 py-1 text-[12.5px] text-white disabled:opacity-50";

interface CardProps {
  title: string;
  which: Which;
  said?: Word;
  trouble?: Word;
  children: React.ReactNode;
}

function Card({ title, which, said, trouble, children }: CardProps) {
  return (
    <section className="mb-3 rounded-[10px] border border-hair px-4 py-3.5">
      <h3 className="mb-0.5 text-[13.5px] font-semibold">{title}</h3>
      {children}
      {trouble?.card === which && <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>}
      {said?.card === which && <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>}
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
