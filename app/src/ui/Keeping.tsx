import { useCallback, useEffect, useState } from "react";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import {
  backUp,
  checked,
  chooseSync,
  reachFor,
  keepSettings,
  reachable,
  rebuild,
  settings as readSettings,
  shortcut,
  restore,
  syncNow,
  syncState,
  type Carrying,
  type Reach,
  type Settings,
  type Reviewed,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { stamped } from "../format";

const carried = { came: "syncCame", same: "syncSame", busy: "syncBusy" } as const;

type Which = "sync" | "backup" | "review" | "terminal" | "quick" | "settings";
type Word = { card: Which; text: string };

interface Props {
  onChanged: () => void;
}

export default function Keeping({ onChanged }: Props) {
  const [state, setState] = useState<Carrying | null>(null);
  const [audit, setAudit] = useState<Reviewed | null>(null);
  const [reach, setReach] = useState<Reach | null>(null);
  const [keys, setKeys] = useState<string | null>(null);
  const [kept, setKept] = useState<Settings | null>(null);
  const [busy, setBusy] = useState<Which | null>(null);
  const [said, setSaid] = useState<Word>();
  // In the card, not the window banner, which would hang over every view.
  const [trouble, setTrouble] = useState<Word>();

  const look = useCallback(() => {
    syncState()
      .then(setState)
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
  }, []);

  useEffect(look, [look]);

  useEffect(() => {
    reachable()
      .then(setReach)
      .catch(() => {});
    shortcut()
      .then(setKeys)
      .catch(() => {});
    readSettings()
      .then(setKept)
      .catch(() => {});
  }, []);

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

  // The early return used to sit above everything, so a failed read left a grey
  // panel with no title, no reason and no button: the screen you go to when
  // something is wrong looked like the app had hung.
  if (!state) {
    return (
      <main className="flex flex-col overflow-hidden">
        <div data-tauri-drag-region className="h-9 shrink-0" />
        <div className="mx-auto w-full max-w-[560px] px-6">
          <h2 className="mb-3.5 text-[21px] font-semibold">{t("keeping")}</h2>
          {trouble && (
            <div className="rounded-xl border border-hair bg-panel p-4">
              <p role="alert" className="text-[12.5px] leading-relaxed text-urgent">
                {trouble.text}
              </p>
              <button
                type="button"
                onClick={() => {
                  setTrouble(undefined);
                  look();
                }}
                className={`mt-2.5 ${strong}`}
              >
                {t("tryAgain")}
              </button>
            </div>
          )}
        </div>
      </main>
    );
  }

  const carrying = busy === "sync";
  // Restoring on top of a running carry is the pair that must never overlap.
  const held = busy !== null;

  // Joining two histories cannot be undone, so the answer is the person's.
  const carryNow = async () => {
    if (held) return;
    setBusy("sync");
    setSaid(undefined);
    setTrouble(undefined);
    try {
      let answer = await syncNow().catch(async (problem) => {
        const refusal = problem as { code?: string; name?: string };
        if (refusal?.code !== "wouldMerge") throw problem;
        if (!(await ask(fill("joinThem", refusal.name ?? ""), { kind: "warning" }))) {
          return "declined" as const;
        }
        return syncNow(undefined, true);
      });

      if (answer === "declined") {
        setTrouble({ card: "sync", text: t("wouldMerge") });
        return;
      }
      setSaid({ card: "sync", text: t(carried[answer]) });
      look();
      onChanged();
    } catch (e) {
      setTrouble({ card: "sync", text: saidPlainly(e) });
    } finally {
      setBusy(null);
    }
  };

  const remember = (next: Settings) =>
    run("settings", keepSettings(next), (now) => {
      setKept(now);
      setSaid({ card: "settings", text: t("settingsKept") });
    });

  const pickFolder = () => {
    if (held) return;
    open({ directory: true })
      .then((at) => typeof at === "string" && run("sync", chooseSync(at), () => {}))
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
  };

  const makeBackup = () => {
    if (held) return;
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
    if (held) return;
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
                <button type="button" disabled={held} onClick={carryNow} className={strong}>
                  {carrying ? t("syncing_") : t("syncNow")}
                </button>
                <button type="button" disabled={held} onClick={pickFolder} className={mild}>
                  {t("changeFolder")}
                </button>
                <button
                  type="button"
                  disabled={held}
                  onClick={() => run("sync", chooseSync(undefined), () => {})}
                  className={mild}
                >
                  {t("syncOffNow")}
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
        </Card>

        <Card title={t("backup")} which="backup" said={said} trouble={trouble}>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {state.backsUp ? t("backupWhat") : t("backupOffWhy")}
          </p>
          {state.backsUp && (
            <div className="mt-2.5 flex items-center gap-2.5">
              <button type="button" disabled={held} onClick={makeBackup} className={mild}>
                {t("backupMake")}
              </button>
              <button type="button" disabled={held} onClick={takeBackup} className={mild}>
                {t("backupRestore")}
              </button>
            </div>
          )}
        </Card>

        {kept && (
          <Card title={t("settingsTitle")} which="settings" said={said} trouble={trouble}>
            <p className="text-[12.5px] leading-relaxed text-soft">{t("noticesWhy")}</p>
            <div className="mt-2.5 flex flex-col gap-1.5">
              {(["screen", "chime"] as const).map((channel) => (
                <label key={channel} className="flex items-center gap-2 text-[12.5px]">
                  <input
                    type="checkbox"
                    checked={!kept.quiet.includes(channel)}
                    disabled={held}
                    onChange={(e) =>
                      remember({
                        ...kept,
                        quiet: e.target.checked
                          ? kept.quiet.filter((one) => one !== channel)
                          : [...kept.quiet, channel],
                      })
                    }
                  />
                  {t(channel === "screen" ? "noticeScreen" : "noticeChime")}
                </label>
              ))}
            </div>

            <p className="mt-3.5 text-[12.5px] leading-relaxed text-soft">{t("attachWhy")}</p>
            <div className="mt-2 flex items-center gap-2.5">
              <select
                aria-label={t("attachUpTo")}
                value={String(kept.attachUpTo)}
                disabled={held}
                onChange={(e) => remember({ ...kept, attachUpTo: Number(e.target.value) })}
                className="rounded-[7px] border border-line bg-bg px-2 py-1 text-[12.5px]"
              >
                {SIZES.map((bytes) => (
                  <option key={bytes} value={bytes}>
                    {weigh(bytes)}
                  </option>
                ))}
              </select>
              <span className="text-[11.5px] text-faint">{t("attachUpTo")}</span>
            </div>
          </Card>
        )}

        <Card title={t("quick")} which="quick" said={said} trouble={trouble}>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {keys ? fill("quickOn", keys) : t("quickNone")}
          </p>
        </Card>

        {reach?.shipped && (
          <Card title={t("terminal")} which="terminal" said={said} trouble={trouble}>
            <p className="text-[12.5px] leading-relaxed text-soft">
              {reach.withinReach ? fill("terminalOn", reach.through ?? reach.at ?? "") : t("terminalOff")}
            </p>
            <div className="mt-2.5 flex items-center gap-2.5">
              <button
                type="button"
                disabled={held}
                onClick={() =>
                  run("terminal", reachFor(!reach.withinReach), (now) => {
                    setReach(now);
                    setSaid({
                      card: "terminal",
                      text: t(now.withinReach ? "terminalFresh" : "terminalGone"),
                    });
                  })
                }
                className={mild}
              >
                {t(reach.withinReach ? "terminalRemove" : "terminalAdd")}
              </button>
            </div>
          </Card>
        )}

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
              disabled={held}
              onClick={() => run("review", checked(), setAudit)}
              className={mild}
            >
              {t("reviewRun")}
            </button>
            {audit && !audit.agrees && (
              <button
                type="button"
                disabled={held}
                onClick={() =>
                  run("review", rebuild().then(checked), (now) => {
                    setAudit(now);
                    setSaid({ card: "review", text: t("reviewRebuilt") });
                  })
                }
                className={strong}
              >
                {t("reviewRedo")}
              </button>
            )}
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
const strong = "rounded-[7px] bg-accent px-2.5 py-1 text-[12.5px] text-bg disabled:opacity-50";

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

/// The band the core will accept, in the steps a person thinks in.
const SIZES = [
  256 * 1024,
  1024 * 1024,
  5 * 1024 * 1024,
  20 * 1024 * 1024,
  50 * 1024 * 1024,
  200 * 1024 * 1024,
];
