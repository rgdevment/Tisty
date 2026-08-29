import { ask, open, save } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import {
  type Ready,
  updateInstall,
  updateReady,
  type Agent,
  agentState,
  agentTurn,
  type About,
  about,
  backUp,
  type Carrying,
  checked,
  chooseSync,
  copied,
  docs,
  facts,
  guide,
  joinThem,
  type Kin,
  keepLocale,
  keepReport,
  keepSettings,
  logs,
  type Machine,
  mergeStores,
  type Reach,
  type Reviewed,
  reachable,
  reachFor,
  settings as readSettings,
  rebuild,
  removeMachine,
  restore,
  retireAttachment,
  revealed,
  type Settings,
  shortcut,
  syncKin,
  syncNow,
  syncState,
  type Twins,
  takeOver,
  twinned,
  type Waking,
  wakeFor,
  waking,
} from "../core";
import { decideAll } from "../deciding";
import { daysFrom, stamped, weigh } from "../format";
import { adopt, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { written } from "../report";
import { type Brittle, scanned } from "../scanning";
import Apart, { type Door } from "./Apart";
import { onMac } from "./WindowChrome";

const carried = {
  came: "syncCame",
  sent: "syncSent",
  both: "syncBoth",
  same: "syncSame",
  busy: "syncBusy",
} as const;

type Which =
  | "sync"
  | "backup"
  | "review"
  | "terminal"
  | "quick"
  | "waking"
  | "settings"
  | "report"
  | "store"
  | "brittle"
  | "greet"
  | "tongue";
type Word = { card: Which; text: string };
type Tab = "data" | "notices" | "writing" | "agents" | "upkeep";

const TABS: { key: Tab; label: Parameters<typeof t>[0] }[] = [
  { key: "data", label: "tabData" },
  { key: "notices", label: "tabNotices" },
  { key: "writing", label: "tabWriting" },
  { key: "agents", label: "tabAgents" },
  { key: "upkeep", label: "tabUpkeep" },
];

interface Props {
  onChanged: () => void;
  onGreet: () => void;
  greeted?: number;
}

export default function Keeping({ onChanged, onGreet, greeted }: Props) {
  const [tab, setTab] = useState<Tab>("data");
  const [agent, setAgent] = useState<Agent | null>(null);
  const [wired, setWired] = useState(false);
  const [looking, setLooking] = useState(false);
  const [found, setFound] = useState<Ready | "none" | null>(null);
  const [asked, setAsked] = useState(false);
  const [state, setState] = useState<Carrying | null>(null);
  const [audit, setAudit] = useState<Reviewed | null>(null);
  const [brittle, setBrittle] = useState<Brittle[] | null>(null);
  const [alike, setAlike] = useState<Twins[] | null>(null);
  const [reach, setReach] = useState<Reach | null>(null);
  const [wake, setWake] = useState<Waking | null>(null);
  const [keys, setKeys] = useState<string | null>(null);
  const [kept, setKept] = useState<Settings | null>(null);
  const [build, setBuild] = useState<About | null>(null);
  const [busy, setBusy] = useState<Which | null>(null);
  const [said, setSaid] = useState<Word>();
  const [trouble, setTrouble] = useState<Word>();
  const [told, setTold] = useState({ names: false, paths: false, logs: true });
  const [paper, setPaper] = useState<string | null>(null);

  const look = useCallback(() => {
    syncState()
      .then(setState)
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
  }, []);

  useEffect(() => {
    if (tab !== "agents") return;
    agentState()
      .then((fresh) => setAgent(fresh))
      .catch((e) => setTrouble({ card: "settings", text: saidPlainly(e) }));
  }, [tab]);

  useEffect(look, [look]);

  useEffect(() => {
    reachable()
      .then(setReach)
      .catch(() => {});
    waking()
      .then(setWake)
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
    look();
  }, [greeted, look]);

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

  const quietly = <T,>(card: Which, work: Promise<T>, then: (answer: T) => void) => {
    setBusy(card);
    setSaid(undefined);
    setTrouble(undefined);
    work
      .then(then)
      .catch((e) => setTrouble({ card, text: saidPlainly(e) }))
      .finally(() => setBusy(null));
  };

  const [apart, setApart] = useState<((door: Door | "else" | null) => void) | null>(null);
  const [kin, setKin] = useState<Kin>("unsure");

  const closed = (door: Door | "else" | null) => {
    apart?.(door);
    setApart(null);
  };

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
  const held = busy !== null;

  const namedDocs = async (files: string[]): Promise<string> => {
    const titled = await docs()
      .then((found) => new Map(found.docs.map((one) => [one.file, one.title])))
      .catch(() => new Map<string, string>());
    return files.map((one) => `«${titled.get(one)?.trim() || t("untitledDoc")}»`).join(", ");
  };

  const carryNow = async (way?: "again"): Promise<"done" | "declined" | "failed"> => {
    if (held) return "failed";
    setBusy("sync");
    setSaid(undefined);
    setTrouble(undefined);
    try {
      const answer = await syncNow(way).catch(async (problem) => {
        const refusal = problem as { code?: string; name?: string };
        if (refusal?.code !== "wouldReset" && refusal?.code !== "otherStore") throw problem;
        setKin(await syncKin().catch(() => "unsure" as const));
        const door = await new Promise<Door | "else" | null>((settle) => setApart(() => settle));
        if (door === null) return "declined" as const;
        if (door === "else") {
          const where = await open({ directory: true });
          if (typeof where !== "string") return "declined" as const;
          await chooseSync(where);
          return syncNow();
        }
        const named = {
          merge: "tisty-before-joining-both",
          mine: "tisty-folder-before",
          theirs: "tisty-before-joining",
        } as const;
        const day = new Date().toISOString().slice(0, 10);
        const at = await save({
          defaultPath: `${named[door]}-${day}.zip`,
          filters: [{ name: "Tisty", extensions: ["zip"] }],
        });
        if (typeof at !== "string") return "declined" as const;
        if (door === "merge") await mergeStores(at);
        else if (door === "mine") await takeOver(at);
        else await joinThem(at);
        return syncNow();
      });

      if (answer === "declined") {
        setTrouble({ card: "sync", text: t("wouldReset") });
        return "declined";
      }
      await decideAll(answer.undecided);
      if (answer.astray?.length) {
        setTrouble({ card: "sync", text: t("someDocsAstray") });
      } else if (answer.unreadable?.length) {
        setTrouble({ card: "sync", text: t("someoneUnreadable") });
      } else if (answer.joined?.length) {
        setSaid({ card: "sync", text: fill("someJoined", await namedDocs(answer.joined)) });
      } else {
        setSaid({ card: "sync", text: t(carried[answer.carried]) });
      }
      look();
      onChanged();
      return "done";
    } catch (e) {
      setTrouble({ card: "sync", text: saidPlainly(e) });
      return "failed";
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
      .then(async (at) => {
        if (typeof at !== "string") return;
        const was = state?.chosen;
        await chooseSync(at);
        look();
        onChanged();
        if ((await carryNow()) !== "declined") return;
        await chooseSync(was);
        look();
        onChanged();
      })
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

  const letGo = (reference: string) => {
    if (held) return;
    ask(fill("looseDropSure", reference.split("/").pop() ?? reference), { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("review", retireAttachment(reference).then(checked), (now) => {
            setAudit(now);
            setSaid({ card: "review", text: t("looseDropped") });
          }),
      )
      .catch((e) => setTrouble({ card: "review", text: saidPlainly(e) }));
  };

  const dropMachine = (one: Machine) => {
    if (held) return;
    const said = `${fill("machineDropSure", one.called)}\n\n${fill(
      "machineDropWhen",
      one.when === 0 ? t("machineNever") : dated(one.when),
    )}`;
    ask(said, { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("review", removeMachine(one.id).then(checked), (now) => {
            setAudit(now);
            setSaid({ card: "review", text: t("machineDropped") });
          }),
      )
      .catch((e) => setTrouble({ card: "review", text: saidPlainly(e) }));
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

  const compose = () => facts(told.names, told.paths).then(written);

  const showReport = () => {
    if (held || paper !== null) return;
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
        return copied(text);
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
      {apart && (
        <Apart
          kin={kin}
          onPick={(door) => closed(door)}
          onElse={() => closed("else")}
          onClose={() => closed(null)}
        />
      )}
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
                    <button
                      type="button"
                      disabled={held}
                      onClick={() => carryNow()}
                      className={strong}
                    >
                      {carrying ? t("syncing_") : t("syncNow")}
                    </button>
                    <button
                      type="button"
                      disabled={held}
                      onClick={() =>
                        state.chosen &&
                        revealed(state.chosen).catch((e) =>
                          setTrouble({ card: "sync", text: saidPlainly(e) }),
                        )
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
                  <p className="mt-2 text-[12px] text-soft">
                    {state.heard ? fill("syncHeard", stamped(state.heard)) : t("syncHeardNever")}
                  </p>
                  {state.heard && -daysFrom(state.heard) >= QUIET_DAYS && (
                    <p className="mt-1.5 text-[12px] leading-relaxed text-ink">
                      {t("syncNothingSince")}
                    </p>
                  )}
                  <p className="mt-1.5 text-[12px] leading-relaxed text-faint">
                    {t("syncOnlyFolder")}
                  </p>
                  <div className="mt-2.5 flex flex-wrap items-center gap-2.5 border-t border-hair pt-2.5">
                    <span className="text-[11.5px] text-faint">{t("syncSetUp")}</span>
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
                    <button
                      type="button"
                      disabled={held}
                      title={t("syncAgainWhy")}
                      onClick={() => carryNow("again")}
                      className={mild}
                    >
                      {t("syncAgain")}
                    </button>
                  </div>
                </>
              )}
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
                <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">
                  {t("noticesMore")}
                </p>
              </Card>
            )}

            <Card title={t("quick")} which="quick" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">
                {keys ? fill("quickOn", keys) : t("quickNone")}
              </p>
            </Card>

            <Card title={t("lookNow")} which="settings" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("lookNowWhen")}</p>
              <div className="mt-2 flex items-center gap-3">
                <button
                  type="button"
                  disabled={held || looking}
                  onClick={() => {
                    setLooking(true);
                    setFound(null);
                    updateReady(true)
                      .then((ready) => setFound(ready ?? "none"))
                      .catch((e) => setTrouble({ card: "settings", text: saidPlainly(e) }))
                      .finally(() => setLooking(false));
                  }}
                  className="rounded-md border border-line px-2.5 py-0.5 text-[12px] text-soft hover:border-accent hover:text-accent disabled:text-faint"
                >
                  {looking ? t("lookingNow") : t("lookNow")}
                </button>
                {found === "none" && (
                  <span className="text-[12.5px] text-soft">{t("lookNowNone")}</span>
                )}
                {found !== null && found !== "none" && (
                  <span className="text-[12.5px] text-soft">
                    {fill("lookNowFound", found.version)}
                  </span>
                )}
              </div>

              {found !== null && found !== "none" && (
                <p className="mt-2 flex flex-wrap items-center gap-2 text-[12.5px] text-soft">
                  {found.installs ? (
                    <button
                      type="button"
                      disabled={asked}
                      onClick={() => {
                        setAsked(true);
                        updateInstall().catch((e) => {
                          setAsked(false);
                          setTrouble({ card: "settings", text: saidPlainly(e) });
                        });
                      }}
                      className="cursor-pointer rounded-lg bg-accent px-2.5 py-1 text-[12px] text-bg disabled:opacity-60"
                    >
                      {t("updateInstall")}
                    </button>
                  ) : found.route === "store" ? (
                    <span className="text-faint">{t("updateStore")}</span>
                  ) : (
                    <code className="text-faint">
                      {fill("updateBrewCli", found.package ?? "tisty")}
                    </code>
                  )}
                </p>
              )}
            </Card>

            {wake?.offered && (
              <Card title={t("wake")} which="waking" busy={busy} said={said} trouble={trouble}>
                <p className="text-[12.5px] leading-relaxed text-soft">
                  {t(wake.wakes ? "wakeOn" : "wakeOff")}
                </p>

                {wake.theirs && !wake.wakes && (
                  <div className="mt-2 rounded-lg bg-mark-priority px-3 py-2.5">
                    <p className="text-[12.5px] leading-relaxed text-ink">{t("wakeTheirs")}</p>
                  </div>
                )}

                <label className="mt-2.5 flex items-center gap-2 text-[12.5px]">
                  <input
                    type="checkbox"
                    checked={wake.wakes}
                    disabled={held}
                    onChange={() =>
                      run("waking", wakeFor(!wake.wakes), (now) => {
                        setWake(now);
                        if (now.wakes === wake.wakes) {
                          return;
                        }
                        setSaid({
                          card: "waking",
                          text: t(now.wakes ? "wakeFresh" : "wakeGone"),
                        });
                      })
                    }
                  />
                  {t("wakeAdd")}
                </label>
              </Card>
            )}

            {kept && (
              <Card title={t("tongue")} which="tongue" busy={busy} said={said} trouble={trouble}>
                <p className="text-[12.5px] leading-relaxed text-soft">{t("tongueWhy")}</p>
                <select
                  aria-label={t("tongue")}
                  value={kept.locale ?? ""}
                  disabled={held}
                  onChange={(e) => {
                    const wanted = e.target.value || undefined;
                    run("tongue", keepLocale(wanted), (now) => {
                      adopt(now ?? undefined);
                      setKept({ ...kept, locale: now ?? undefined });
                      onChanged();
                    });
                  }}
                  className={`mt-2.5 rounded-[7px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
                >
                  <option value="">{t("tongueTheirs")}</option>
                  <option value="es">Español</option>
                  <option value="en">English</option>
                </select>
              </Card>
            )}

            <Card title={t("greetAgain")} which="greet" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("greetAgainWhy")}</p>
              <div className="mt-2.5 flex flex-wrap items-center gap-2.5">
                <button type="button" onClick={onGreet} className={mild}>
                  {t("greetAgainDo")}
                </button>
                <button
                  type="button"
                  disabled={held}
                  onClick={() =>
                    run("greet", guide(), () => {
                      setSaid({ card: "greet", text: t("guideKept") });
                      onChanged();
                    })
                  }
                  className={mild}
                >
                  {t("welcomeGuide")}
                </button>
              </div>
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
                <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">{t("attachBig")}</p>
              </Card>
            )}

            {reach?.shipped && (
              <Card
                title={t("terminal")}
                which="terminal"
                busy={busy}
                said={said}
                trouble={trouble}
              >
                <p className="text-[12.5px] leading-relaxed text-soft">
                  {reach.withinReach
                    ? fill("terminalOn", reach.through ?? reach.at ?? "")
                    : t("terminalOff")}
                </p>

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
                          text: t(
                            now.withinReach
                              ? onMac
                                ? "terminalFreshNow"
                                : "terminalFresh"
                              : "terminalGone",
                          ),
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

        {tab === "agents" && (
          <>
            <Group label={t("agentsTitle")} />

            <Card
              title={t("agentsTitle")}
              which="settings"
              busy={busy}
              said={said}
              trouble={trouble}
            >
              <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsWhat")}</p>

              <div className="mt-3 flex items-center justify-between gap-3 rounded-lg border border-hair px-3 py-2.5">
                <span className="min-w-0">
                  <span className="block text-[13px] font-semibold">
                    {agent?.on ? fill("agentsOn", agent.called ?? "") : t("agentsOff")}
                  </span>
                  {agent?.on && (
                    <>
                      <span className="block text-[12px] text-soft">
                        {agent.filed > 0
                          ? fill("agentsFiled", String(agent.filed))
                          : t("agentsFiledNone")}
                      </span>
                      <span className="block font-mono text-[10.5px] break-all text-faint">
                        {agent.id}
                      </span>
                    </>
                  )}
                </span>
                <button
                  type="button"
                  disabled={held}
                  onClick={() => {
                    agentTurn(!agent?.on)
                      .then((fresh) => setAgent(fresh))
                      .catch((e) => setTrouble({ card: "settings", text: saidPlainly(e) }));
                  }}
                  className={`shrink-0 rounded-md border px-2.5 py-1 text-[12px] disabled:text-faint ${
                    agent?.on
                      ? "border-line text-soft hover:border-urgent hover:text-urgent"
                      : "border-accent text-accent"
                  }`}
                >
                  {agent?.on ? t("agentsTurnOff") : t("agentsTurnOn")}
                </button>
              </div>

              <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("agentsUndo")}</p>
            </Card>

            <Card
              title={t("agentsCanTitle")}
              which="settings"
              busy={busy}
              said={said}
              trouble={trouble}
            >
              <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsCan")}</p>
            </Card>

            <Card
              title={t("agentsCannotTitle")}
              which="settings"
              busy={busy}
              said={said}
              trouble={trouble}
            >
              <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsCannot")}</p>
            </Card>

            <Card
              title={t("agentsHowTitle")}
              which="settings"
              busy={busy}
              said={said}
              trouble={trouble}
            >
              <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsHow")}</p>
              <pre className="mt-2 overflow-x-auto rounded-lg border border-hair px-3 py-2 font-mono text-[11.5px] text-soft">
                {WIRING}
              </pre>
              <button
                type="button"
                onClick={() => {
                  void copied(WIRING).then(() => {
                    setWired(true);
                    window.setTimeout(() => setWired(false), 1500);
                  });
                }}
                className="mt-2 rounded-md border border-line px-2.5 py-0.5 text-[12px] text-soft hover:border-accent hover:text-accent"
              >
                {wired ? t("agentsCopied") : t("agentsCopy")}
              </button>
            </Card>
          </>
        )}

        {tab === "upkeep" && (
          <>
            <Group label={t("theStore")} />

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

            <Group label={t("theMachines")} />

            <Card title={t("theMachines")} which="review" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("machinesWhat")}</p>
              {audit && (
                <ul className="mt-2 flex flex-col gap-1 text-[12.5px]">
                  {audit.machines.map((one) => (
                    <li
                      key={one.id}
                      className={`flex items-center justify-between gap-3 rounded-lg px-2.5 py-2 ${
                        one.mine ? "bg-accent-soft" : ""
                      }`}
                    >
                      <span className="min-w-0">
                        <span className="block text-[13.5px] font-semibold">
                          {one.called}
                          {one.mine && (
                            <span className="ml-2 rounded-full border border-accent px-1.5 py-px align-[1px] text-[10.5px] font-semibold tracking-wide text-accent uppercase">
                              {t("machineHere")}
                            </span>
                          )}
                        </span>
                        <span
                          className={`block text-[12px] ${hushed(one) ? "text-ink" : "text-soft"}`}
                        >
                          {one.when === 0 ? t("machineNever") : dated(one.when)}
                        </span>
                        <span className="block font-mono text-[10.5px] break-all text-faint">
                          {one.id}
                        </span>
                      </span>
                      <span className="flex shrink-0 items-center gap-2.5">
                        {one.mine ? (
                          <span className="text-[12px] text-faint">{t("machineNeverDrop")}</span>
                        ) : (
                          <button
                            type="button"
                            disabled={held}
                            onClick={() => dropMachine(one)}
                            className="rounded-md border border-line px-2.5 py-0.5 text-[12px] text-soft hover:border-urgent hover:text-urgent disabled:text-faint"
                          >
                            {t("machineDrop")}
                          </button>
                        )}
                      </span>
                    </li>
                  ))}
                </ul>
              )}
              {audit?.machines.some(hushed) && (
                <p className="mt-2 text-[12.5px] leading-relaxed text-soft">{t("machineHushed")}</p>
              )}
              {audit && audit.machines.length === 0 && (
                <p className="mt-2 text-[12.5px] text-faint">{t("machinesNone")}</p>
              )}
            </Card>

            <Group label={t("looseAre")} />

            <Card title={t("looseAre")} which="review" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("looseWhat")}</p>
              {audit?.machines.some(hushed) && (
                <p className="mt-1.5 text-[12.5px] leading-relaxed text-soft">{t("looseWait")}</p>
              )}
              {audit && audit.loose === 0 && (
                <p className="mt-2 text-[12.5px] text-faint">{t("looseNone")}</p>
              )}
              {audit && audit.stranded > 0 && (
                <p className="mt-2 text-[12.5px] leading-relaxed text-urgent">
                  {fill("strandedPapers", String(audit.stranded))}
                </p>
              )}
              {audit && audit.loose > 0 && (
                <>
                  <p className="mt-2 text-[12.5px] tabular-nums text-soft">
                    {`${fill("looseTotal", String(audit.loose))} · ${weigh(audit.looseBytes)}`}
                  </p>
                  <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
                    {audit.astray.map((one) => (
                      <li key={one.at} className="flex items-baseline justify-between gap-4">
                        <span className="font-mono text-[11.5px] break-all text-soft">
                          {one.at.split("/").pop()}
                        </span>
                        <span className="flex shrink-0 items-baseline gap-2.5 tabular-nums">
                          <span className="text-faint">
                            {`${weigh(one.bytes)} · ${dated(one.when)}`}
                          </span>
                          <button
                            type="button"
                            disabled={held}
                            onClick={() => letGo(one.at)}
                            className="text-[11.5px] text-urgent hover:underline disabled:text-soft"
                          >
                            {t("looseDrop")}
                          </button>
                        </span>
                      </li>
                    ))}
                  </ul>
                  <div className="mt-2.5 flex items-center gap-2.5">
                    <button
                      type="button"
                      disabled={!build}
                      onClick={() =>
                        build &&
                        revealed(build.store).catch((e) =>
                          setTrouble({ card: "review", text: saidPlainly(e) }),
                        )
                      }
                      className={mild}
                    >
                      {t("aboutReveal")}
                    </button>
                  </div>
                </>
              )}
            </Card>

            <Group label={t("twinsAre")} />

            <Card title={t("twinsAre")} which="review" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("twinsWhat")}</p>
              {alike?.length === 0 && (
                <p className="mt-2 text-[12.5px] text-faint">{t("twinsNone")}</p>
              )}
              {alike && alike.length > 0 && (
                <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-2 overflow-y-auto text-[12.5px]">
                  {alike.map((one) => (
                    <li key={one.at.join("|")}>
                      <span className="tabular-nums text-faint">{weigh(one.bytes)}</span>
                      {one.at.map((named) => (
                        <span
                          key={named}
                          className="block font-mono text-[11.5px] break-all text-soft"
                        >
                          {named.replace("attachments/", "")}
                        </span>
                      ))}
                    </li>
                  ))}
                </ul>
              )}
              <div className="mt-2.5 flex items-center gap-2.5">
                <button
                  type="button"
                  disabled={held}
                  onClick={() => run("review", twinned(), setAlike)}
                  className={mild}
                >
                  {t(alike ? "twinsAgain" : "twinsRun")}
                </button>
              </div>
            </Card>

            <Group label={t("brittleAre")} />

            <Card title={t("brittleAre")} which="brittle" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("brittleWhat")}</p>
              {brittle?.length === 0 && (
                <p className="mt-2 text-[12.5px] text-faint">{t("brittleNone")}</p>
              )}
              {brittle && brittle.length > 0 && (
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
              <div className="mt-2.5 flex items-center gap-2.5">
                <button
                  type="button"
                  disabled={held}
                  onClick={() => run("brittle", scanned(), setBrittle)}
                  className={mild}
                >
                  {t(brittle ? "brittleAgain" : "brittleRun")}
                </button>
              </div>
            </Card>

            <Group label={t("reportTitle")} />

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

const HUSHED = 7 * 24 * 60 * 60;
const QUIET_DAYS = 3;

const hushed = (one: Machine): boolean =>
  !one.mine && (one.when === 0 || Date.now() / 1000 - one.when > HUSHED);

const dated = (when: number): string => {
  const at = new Date(when * 1000);
  return Number.isNaN(at.getTime()) ? "—" : stamped(at.toISOString());
};

const off = "disabled:border-hair disabled:bg-hair disabled:text-soft";
const mild = `rounded-[7px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover ${off}`;
const strong = `rounded-[7px] bg-accent px-2.5 py-1 text-[12.5px] text-bg ${off}`;
const risky = `rounded-[7px] border border-urgent/45 px-2.5 py-1 text-[12.5px] text-urgent hover:bg-urgent/10 ${off}`;

const WIRING = `{
  "mcpServers": {
    "tisty": { "command": "tisty", "args": ["mcp"] }
  }
}`;

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

const NAMED: Record<Which, Parameters<typeof t>[0]> = {
  sync: "syncing",
  backup: "backup",
  review: "review",
  brittle: "brittleAre",
  terminal: "terminal",
  quick: "quick",
  waking: "wake",
  greet: "greetAgain",
  tongue: "tongue",
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
      {waiting && (
        <p className="mt-2 text-[11.5px] text-faint">{fill("waitFor", t(NAMED[busy]))}</p>
      )}
      {trouble?.card === which && <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>}
      {said?.card === which && <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>}
    </section>
  );
}

const SIZES = [256 * 1024, 1024 * 1024, 5 * 1024 * 1024, 20 * 1024 * 1024, 50 * 1024 * 1024];
