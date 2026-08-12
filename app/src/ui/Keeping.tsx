import { useCallback, useEffect, useState } from "react";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import {
  about,
  backUp,
  checked,
  chooseSync,
  facts,
  keepReport,
  logs,
  reachFor,
  keepSettings,
  reachable,
  rebuild,
  revealed,
  settings as readSettings,
  shortcut,
  restore,
  syncNow,
  syncState,
  type About,
  type Carrying,
  type Reach,
  type Settings,
  type Reviewed,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { stamped, weigh } from "../format";
import { written } from "../report";

const carried = { came: "syncCame", same: "syncSame", busy: "syncBusy" } as const;

type Which =
  | "sync"
  | "backup"
  | "review"
  | "terminal"
  | "quick"
  | "settings"
  | "report"
  | "store";
type Word = { card: Which; text: string };
type Tab = "data" | "notices" | "writing" | "upkeep";

const TABS: { key: Tab; label: Parameters<typeof t>[0] }[] = [
  { key: "data", label: "tabData" },
  { key: "notices", label: "tabNotices" },
  { key: "writing", label: "tabWriting" },
  { key: "upkeep", label: "tabUpkeep" },
];

interface Props {
  onChanged: () => void;
}

export default function Keeping({ onChanged }: Props) {
  const [tab, setTab] = useState<Tab>("data");
  const [state, setState] = useState<Carrying | null>(null);
  const [audit, setAudit] = useState<Reviewed | null>(null);
  const [reach, setReach] = useState<Reach | null>(null);
  const [keys, setKeys] = useState<string | null>(null);
  const [kept, setKept] = useState<Settings | null>(null);
  const [build, setBuild] = useState<About | null>(null);
  const [busy, setBusy] = useState<Which | null>(null);
  const [said, setSaid] = useState<Word>();
  // In the card, not the window banner, which would hang over every view.
  const [trouble, setTrouble] = useState<Word>();
  const [told, setTold] = useState({ names: false, paths: false, logs: true });
  const [paper, setPaper] = useState<string | null>(null);

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
    about()
      .then(setBuild)
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

  /// Reads and changes nothing, so the window behind it is not reloaded.
  const quietly = <T,>(card: Which, work: Promise<T>, then: (answer: T) => void) => {
    setBusy(card);
    setSaid(undefined);
    setTrouble(undefined);
    work
      .then(then)
      .catch((e) => setTrouble({ card, text: saidPlainly(e) }))
      .finally(() => setBusy(null));
  };

  // Below the title and the retry: a failed read used to leave a grey panel
  // with nothing to press.
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

  // Counting every event is not free, so it waits to be asked for.
  const compose = () => facts(told.names, told.paths).then(written);

  const showReport = () => {
    if (held || paper !== null) return;
    // What is on screen is what goes in the file, log included: «it is redacted»
    // is a promise, and the text underneath is the proof.
    quietly(
      "report",
      Promise.all([compose(), told.logs ? logs(TAIL) : Promise.resolve(null)]).then(
        ([text, kept]) => (kept ? `${text}\n${LOGS}\n${kept.lines.join("\n")}\n` : text),
      ),
      setPaper,
    );
  };

  const changeTold = (next: typeof told) => {
    setTold(next);
    // Or the screen would show one redaction while saving another.
    setPaper(null);
  };

  const saveReport = () => {
    if (held) return;
    setSaid(undefined);
    setTrouble(undefined);
    Promise.all([
      save({ defaultPath: "tisty-report.zip", filters: [{ name: "Tisty", extensions: ["zip"] }] }),
      paper !== null ? Promise.resolve(paper) : compose(),
    ])
      .then(([at, text]) => {
        setPaper(text);
        if (typeof at !== "string") return;
        quietly("report", keepReport(at, text, told.logs), () =>
          setSaid({ card: "report", text: fill("reportKept", at) }),
        );
      })
      .catch((e) => setTrouble({ card: "report", text: saidPlainly(e) }));
  };

  const copyReport = () => {
    if (held) return;
    (paper !== null ? Promise.resolve(paper) : compose())
      .then((text) => {
        setPaper(text);
        return navigator.clipboard.writeText(text);
      })
      .then(() => setSaid({ card: "report", text: t("reportCopied") }))
      .catch(() => setTrouble({ card: "report", text: t("reportNoClipboard") }));
  };

  const holds = [
    fill("openTasks", String(state.open)),
    fill("archivedTasks", String(state.archived)),
    fill("reviewLists", String(state.lists)),
    fill("someAttachments", String(state.attachments)),
  ].join(" · ");

  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="scroller mx-auto w-full max-w-[560px] px-6 pb-12">
        <h2 className="mb-3.5 text-[21px] font-semibold">{t("keeping")}</h2>

        <div role="tablist" className="mb-4 flex flex-wrap gap-1">
          {TABS.map((one) => (
            <button
              key={one.key}
              type="button"
              role="tab"
              aria-selected={tab === one.key}
              onClick={() => setTab(one.key)}
              className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
                tab === one.key
                  ? "border-ink bg-ink text-bg"
                  : "border-line text-faint hover:text-soft"
              }`}
            >
              {t(one.label)}
            </button>
          ))}
        </div>

        {tab === "data" && (
          <>
            <Card title={t("syncing")} which="sync" busy={busy} said={said} trouble={trouble}>
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

            <Group label={t("backup")} />

            {state.backsUp ? (
              <>
                <Card
                  title={t("backupSave")}
                  which="backup"
                  busy={busy}
                  said={said}
                  trouble={trouble}
                >
                  <p className="text-[12.5px] leading-relaxed text-soft">{t("backupWhat")}</p>
                  <dl className="mt-2 grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-0.5 text-[12.5px]">
                    <dt className="text-faint">{t("backupHolds")}</dt>
                    <dd className="text-soft">{holds}</dd>
                    <dt className="text-faint">{t("backupWeighs")}</dt>
                    <dd className="tabular-nums text-soft">
                      {fill("backupAbout", weigh(state.weight))}
                    </dd>
                    <dt className="text-faint">{t("backupLast")}</dt>
                    <dd className="text-soft">
                      {state.backedUpAt ? stamped(state.backedUpAt) : t("backupNever")}
                    </dd>
                  </dl>
                  <div className="mt-2.5 flex items-center gap-2.5">
                    <button type="button" disabled={held} onClick={makeBackup} className={strong}>
                      {t("backupMake")}
                    </button>
                  </div>
                </Card>

                {/* Its own card: one of the two replaces everything you have. */}
                <section className="mb-3 rounded-[10px] border border-urgent/35 bg-urgent/8 px-4 py-3.5">
                  <h3 className="mb-0.5 text-[13.5px] font-semibold">{t("restoreTitle")}</h3>
                  <p className="text-[12.5px] leading-relaxed text-urgent">{t("restoreWhat")}</p>
                  <div className="mt-2.5 flex items-center gap-2.5">
                    <button type="button" disabled={held} onClick={takeBackup} className={risky}>
                      {t("restoreFrom")}
                    </button>
                  </div>
                </section>
              </>
            ) : (
              <Card title={t("backup")} which="backup" busy={busy} said={said} trouble={trouble}>
                <p className="text-[12.5px] leading-relaxed text-soft">{t("backupOffWhy")}</p>
              </Card>
            )}

            <Group label={t("whereItLives")} />

            <Card title={t("aboutStore")} which="store" busy={busy} said={said} trouble={trouble}>
              <p className="font-mono text-[11.5px] leading-relaxed break-all text-soft">
                {build?.store ?? "…"}
              </p>
              <p className="mt-1.5 text-[11.5px] leading-relaxed text-faint">{t("storeFixed")}</p>
              <div className="mt-2.5 flex items-center gap-2.5">
                <button
                  type="button"
                  disabled={!build}
                  onClick={() =>
                    build &&
                    revealed(build.store).catch((e) =>
                      setTrouble({ card: "store", text: saidPlainly(e) }),
                    )
                  }
                  className={mild}
                >
                  {t("aboutReveal")}
                </button>
              </div>
            </Card>
          </>
        )}

        {tab === "notices" && (
          <>
            {kept && (
              <Card
                title={t("settingsTitle")}
                which="settings"
                busy={busy}
                said={said}
                trouble={trouble}
              >
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
                <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">{t("noticesMore")}</p>
              </Card>
            )}

            <Card title={t("quick")} which="quick" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">
                {keys ? fill("quickOn", keys) : t("quickNone")}
              </p>
            </Card>
          </>
        )}

        {tab === "writing" && (
          <>
            {kept && (
              <Card
                title={t("attachTitle")}
                which="settings"
                busy={busy}
                said={said}
                trouble={trouble}
              >
                <p className="text-[12.5px] leading-relaxed text-soft">{t("attachWhy")}</p>
                <div className="mt-2 flex items-center gap-2.5">
                  <select
                    aria-label={t("attachUpTo")}
                    value={String(kept.attachUpTo)}
                    disabled={held}
                    onChange={(e) => remember({ ...kept, attachUpTo: Number(e.target.value) })}
                    className={`rounded-[7px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
                  >
                    {SIZES.map((bytes) => (
                      <option key={bytes} value={bytes}>
                        {weigh(bytes)}
                      </option>
                    ))}
                  </select>
                  <span className="text-[11.5px] text-faint">{t("attachUpTo")}</span>
                </div>
                <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">
                  {t("attachBig")} <span className="text-high">{t("docsSoon")}</span>
                </p>
              </Card>
            )}

            {reach?.shipped && (
              <Card title={t("terminal")} which="terminal" busy={busy} said={said} trouble={trouble}>
                <p className="text-[12.5px] leading-relaxed text-soft">
                  {reach.withinReach
                    ? fill("terminalOn", reach.through ?? reach.at ?? "")
                    : t("terminalOff")}
                </p>

                {/* «Done» and «works» are different answers: macOS builds its
                    PATH from /etc/paths, which does not include ~/.local/bin. */}
                {reach.withinReach && !reach.onPath && (
                  <div className="mt-2 rounded-lg bg-mark-priority px-3 py-2.5">
                    <p className="text-[12.5px] leading-relaxed text-ink">
                      {t("terminalNotOnPath")}
                    </p>
                    <code className="mt-1.5 block font-mono text-[11.5px] break-all text-soft">
                      export PATH=&quot;$HOME/.local/bin:$PATH&quot;
                    </code>
                    <p className="mt-1.5 text-[11.5px] leading-relaxed text-faint">
                      {t("terminalOrBrew")}
                    </p>
                  </div>
                )}

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
          </>
        )}

        {tab === "upkeep" && (
          <>
            <Card title={t("review")} which="review" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("reviewWhat")}</p>
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
                  <dt className="text-faint">{t("looseAre")}</dt>
                  <dd className="tabular-nums text-soft">
                    {audit.loose === 0 ? "0" : `${audit.loose} · ${weigh(audit.looseBytes)}`}
                  </dd>
                  <dt className="text-faint">{t("devicesAre")}</dt>
                  <dd className="tabular-nums text-soft">{audit.devices}</dd>
                </dl>
              )}
              <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
                <button
                  type="button"
                  disabled={held}
                  onClick={() => run("review", checked(), setAudit)}
                  className={mild}
                >
                  {t(audit ? "reviewAgain" : "reviewRun")}
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
              </div>
            </Card>

            <Card title={t("reportTitle")} which="report" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">
                {t("reportWhat")} <span className="text-ink">{t("reportNeverSent")}</span>{" "}
                {t("reportYours")}
              </p>

              <div className="mt-2.5 flex flex-col gap-2">
                <label className="flex items-start gap-2 text-[12.5px]">
                  <input
                    type="checkbox"
                    checked={told.logs}
                    disabled={held}
                    onChange={(e) => changeTold({ ...told, logs: e.target.checked })}
                    className="mt-0.5"
                  />
                  <span>
                    {t("reportLogs")}
                    <span className="block text-[11.5px] text-faint">{t("reportLogsWhy")}</span>
                  </span>
                </label>
                <label className="flex items-start gap-2 text-[12.5px]">
                  <input
                    type="checkbox"
                    checked={told.names}
                    disabled={held}
                    onChange={(e) => changeTold({ ...told, names: e.target.checked })}
                    className="mt-0.5"
                  />
                  <span>
                    {t("reportNames")}
                    <span className="block text-[11.5px] text-faint">{t("reportNamesWhy")}</span>
                  </span>
                </label>
                <label className="flex items-start gap-2 text-[12.5px]">
                  <input
                    type="checkbox"
                    checked={told.paths}
                    disabled={held}
                    onChange={(e) => changeTold({ ...told, paths: e.target.checked })}
                    className="mt-0.5"
                  />
                  <span>
                    {t("reportPaths")}
                    <span className="block text-[11.5px] text-faint">{t("reportPathsWhy")}</span>
                  </span>
                </label>
              </div>

              <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">{t("reportNever")}</p>

              <details className="mt-2.5" onToggle={showReport}>
                <summary className="cursor-pointer text-[12.5px] text-accent">
                  {t("reportShow")}
                </summary>
                <pre className="scroller mt-2 max-h-[22rem] overflow-x-auto rounded-lg bg-hover px-3 py-2.5 font-mono text-[11.5px] leading-relaxed text-soft">
                  {paper ?? "…"}
                </pre>
              </details>

              <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
                <button type="button" disabled={held} onClick={saveReport} className={strong}>
                  {t("reportSave")}
                </button>
                <button type="button" disabled={held} onClick={copyReport} className={mild}>
                  {t("reportCopy")}
                </button>
              </div>
            </Card>

          </>
        )}
      </div>
    </main>
  );
}

// Not `opacity-50`: half-strength ink lands at 2:1, and no test can measure a
// colour that is only ever computed by the compositor.
const off = "disabled:border-hair disabled:bg-hair disabled:text-soft";
const mild = `rounded-[7px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover ${off}`;
const strong = `rounded-[7px] bg-accent px-2.5 py-1 text-[12.5px] text-bg ${off}`;
const risky = `rounded-[7px] border border-urgent/45 px-2.5 py-1 text-[12.5px] text-urgent hover:bg-urgent/10 ${off}`;

function Group({ label }: { label: string }) {
  return (
    <div className="mt-5 mb-2 flex items-center gap-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
      <span>{label}</span>
      <span className="h-px flex-1 bg-hair" />
    </div>
  );
}

interface CardProps {
  title: string;
  which: Which;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  children: React.ReactNode;
}

/// Work anywhere holds every button here, so every card names what it waits on.
const NAMED: Record<Which, Parameters<typeof t>[0]> = {
  sync: "syncing",
  backup: "backup",
  review: "review",
  terminal: "terminal",
  quick: "quick",
  settings: "settingsTitle",
  report: "reportTitle",
  store: "aboutStore",
};

const TAIL = 300;
const LOGS = "\n--- tisty.log ---";

function Card({ title, which, busy, said, trouble, children }: CardProps) {
  const waiting = busy !== null && busy !== which;
  return (
    <section className="mb-3 rounded-[10px] border border-hair px-4 py-3.5">
      <h3 className="mb-0.5 text-[13.5px] font-semibold">{title}</h3>
      {children}
      {waiting && <p className="mt-2 text-[11.5px] text-faint">{fill("waitFor", t(NAMED[busy]))}</p>}
      {trouble?.card === which && <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>}
      {said?.card === which && <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>}
    </section>
  );
}

const SIZES = [
  256 * 1024,
  1024 * 1024,
  5 * 1024 * 1024,
  20 * 1024 * 1024,
  50 * 1024 * 1024,
  200 * 1024 * 1024,
];
