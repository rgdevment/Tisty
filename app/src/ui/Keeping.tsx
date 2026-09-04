import { listen } from "@tauri-apps/api/event";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import {
  type About,
  type Agent,
  type Astray,
  about,
  agentState,
  agentTurn,
  backUp,
  type Carrying,
  checked,
  chooseSync,
  copied,
  docAdopt,
  docDrop,
  docLetGo,
  docs,
  type Freeing,
  facts,
  freeUp,
  type Gone,
  guide,
  type Holds,
  joinThem,
  type Keeper,
  type Kin,
  keepLocale,
  keepReport,
  keepSettings,
  logs,
  type Machine,
  mergeStores,
  type Reach,
  type Ready,
  type Reviewed,
  reachable,
  reachFor,
  settings as readSettings,
  rebuild,
  removeMachine,
  restore,
  retireAttachment,
  retireAttachments,
  revealed,
  type Settings,
  type Stray,
  seenAgents,
  shortcut,
  stopFreeing,
  syncKin,
  syncNow,
  syncState,
  type Twins,
  takeOver,
  twinned,
  unwireAgent,
  updateInstall,
  updateReady,
  type Waking,
  type Wired,
  wakeFor,
  waking,
  wireAgent,
} from "../core";
import { decideAll } from "../deciding";
import { daysFrom, stamped, weigh } from "../format";
import { warningOf } from "../keepers";
import { adopt, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { written } from "../report";
import { type Brittle, scanned } from "../scanning";
import Apart, { type Door } from "./Apart";
import Keepers from "./Keepers";
import Modal from "./Modal";
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
  | "restore"
  | "review"
  | "terminal"
  | "quick"
  | "waking"
  | "settings"
  | "notices"
  | "updates"
  | "attach"
  | "holds"
  | "wiring"
  | "report"
  | "store"
  | "brittle"
  | "greet"
  | "tongue";
type Word = { card: Which; text: string };
type Tab = "general" | "data" | "agents" | "upkeep";

const TABS: { key: Tab; label: Parameters<typeof t>[0] }[] = [
  { key: "general", label: "tabGeneral" },
  { key: "data", label: "tabData" },
  { key: "agents", label: "tabAgents" },
  { key: "upkeep", label: "tabUpkeep" },
];

interface Props {
  onChanged: () => void;
  onGreet: () => void;
  onDoc: (paper: string) => void;
  greeted?: number;
}

export default function Keeping({ onChanged, onGreet, onDoc, greeted }: Props) {
  const [tab, setTab] = useState<Tab>("general");
  const [agent, setAgent] = useState<Agent | null>(null);
  const [agents, setAgents] = useState<Wired[] | null>(null);
  const [wired, setWired] = useState(false);
  const [typed, setTyped] = useState(false);
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
  const [freeing, setFreeing] = useState<Freeing | null>(null);

  useEffect(() => {
    const off = listen<Freeing>("freeing", (told) => setFreeing(told.payload));
    return () => {
      void off.then((stop) => stop());
    };
  }, []);
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
    readSettings()
      .then(setKept)
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (tab !== "agents") return;
    agentState()
      .then((fresh) => setAgent(fresh))
      .catch((e) => setTrouble({ card: "settings", text: saidPlainly(e) }));
    seenAgents()
      .then(setAgents)
      .catch((e) => setTrouble({ card: "wiring", text: saidPlainly(e) }));
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

  const join = (one: Wired) => {
    const out = one.wired && !one.astray;
    quietly("wiring", out ? unwireAgent(one.id) : wireAgent(one.id), (now) => {
      setAgents(now);
      setSaid({ card: "wiring", text: t(out ? "wiringGone" : "wiringFresh") });
    });
  };

  const [picking, setPicking] = useState(false);
  const [was, setWas] = useState<string>();
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
      const shut = await decideAll(answer.undecided);
      if (answer.astray?.length) {
        setTrouble({ card: "sync", text: t("someDocsAstray") });
      } else if (answer.unreadable?.length) {
        setTrouble({ card: "sync", text: t("someoneUnreadable") });
      } else if (shut.length) {
        setTrouble({ card: "sync", text: fill("someLockedAtOdds", await namedDocs(shut)) });
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

  const remember = (next: Settings, card: Which) =>
    run(card, keepSettings(next), (now) => {
      setKept(now);
      setSaid({ card, text: t("settingsKept") });
    });

  const pickFolder = () => {
    if (held) return;
    setWas(state?.chosen);
    setPicking(true);
  };

  const picked = async (at?: string) => {
    setPicking(false);
    look();
    onChanged();
    if (!at) return;
    if ((await carryNow()) !== "declined") return;
    await chooseSync(was).catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
    look();
    onChanged();
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

  const letGoOfAll = (astray: Astray[]) => {
    if (held || astray.length === 0) return;
    ask(fill("upkeepSafeAllSure", String(astray.length)), { kind: "warning" })
      .then(async (sure) => {
        if (!sure) return;
        run("review", retireAttachments(astray.map((one) => one.at)).then(checked), (now) => {
          setAudit(now);
          setSaid({ card: "review", text: t("looseDropped") });
        });
      })
      .catch((e) => setTrouble({ card: "review", text: saidPlainly(e) }));
  };

  const forgetMissing = (one: Gone) => {
    if (held) return;
    ask(fill("dropDocSure", one.title || one.file), { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("review", docDrop(one.id).then(checked), (now) => {
            setAudit(now);
            setSaid({ card: "review", text: t("looseDropped") });
          }),
      )
      .catch((e) => setTrouble({ card: "review", text: saidPlainly(e) }));
  };

  const takeInAll = (strays: Stray[]) =>
    run(
      "review",
      strays
        .reduce((so, one) => so.then(() => docAdopt(one.file).then(() => {})), Promise.resolve())
        .then(checked),
      setAudit,
    );

  const takeIn = (file: string) =>
    run(
      "review",
      docAdopt(file).then(async (made) => ({ made, now: await checked() })),
      (both) => {
        setAudit(both.now);
        setSaid({ card: "review", text: fill("upkeepTakenIn", both.made.title || both.made.id) });
      },
    );

  const letGoOfPaper = (one: Stray) => {
    if (held) return;
    ask(fill("upkeepDropSure", one.title || one.file), { kind: "warning" })
      .then(
        (sure) =>
          sure &&
          run("review", docLetGo(one.file).then(checked), (now) => {
            setAudit(now);
            setSaid({ card: "review", text: t("upkeepDropped") });
          }),
      )
      .catch((e) => setTrouble({ card: "review", text: saidPlainly(e) }));
  };

  const letGo = (reference: string, shared?: boolean) => {
    if (held) return;
    const named = reference.split("/").pop() ?? reference;
    ask(fill(shared ? "looseDropSharedSure" : "looseDropSure", named), { kind: "warning" })
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
        run("restore", restore(at), (files) =>
          setSaid({ card: "restore", text: fill("restored", String(files)) }),
        );
      })
      .catch((e) => setTrouble({ card: "restore", text: saidPlainly(e) }));
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
      {picking && (
        <Modal title={t("welcomeCopies")} wide onClose={() => setPicking(false)}>
          <p className="mb-4 text-[12.5px] leading-relaxed text-soft">{t("keepersWhy")}</p>
          <Keepers
            busy={held}
            onTrouble={(text) => setTrouble({ card: "sync", text })}
            onDone={picked}
          />
        </Modal>
      )}
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

        {busy !== null && (tab === "general" || tab === "data") && (
          <p className="text-[11.5px] text-faint">{fill("waitFor", t(NAMED[busy]))}</p>
        )}

        {tab === "general" && (
          <>
            <Band label={t("bandWindow")} />
            <div className="border-t border-hair">
              {kept && (
                <Line
                  title={t("tongue")}
                  why={t("tongueWhy")}
                  which="tongue"
                  said={said}
                  trouble={trouble}
                >
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
                    className={`rounded-[7px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
                  >
                    <option value="">{t("tongueTheirs")}</option>
                    <option value="es">Español</option>
                    <option value="en">English</option>
                  </select>
                </Line>
              )}

              {wake?.offered && (
                <Line
                  title={t("wake")}
                  why={t(wake.wakes ? "wakeOn" : "wakeOff")}
                  which="waking"
                  said={said}
                  trouble={trouble}
                  more={
                    wake.theirs &&
                    !wake.wakes && (
                      <div className="mt-2 rounded-lg bg-mark-priority px-3 py-2.5">
                        <p className="text-[12.5px] leading-relaxed text-ink">{t("wakeTheirs")}</p>
                      </div>
                    )
                  }
                >
                  <Knob
                    on={wake.wakes}
                    label={t("wakeAdd")}
                    disabled={held}
                    onPress={() =>
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
                </Line>
              )}

              <Line
                title={t("quick")}
                why={keys ? fill("quickOn", keys) : t("quickNone")}
                which="quick"
                said={said}
                trouble={trouble}
              />
            </div>

            <Band label={t("bandNotices")} />
            <div className="border-t border-hair">
              {kept &&
                (["screen", "chime"] as const).map((channel) => (
                  <Line
                    key={channel}
                    title={t(channel === "screen" ? "noticeScreen" : "noticeChime")}
                    why={channel === "screen" ? t("noticesWhy") : undefined}
                    which="notices"
                    said={said}
                    trouble={trouble}
                  >
                    <Knob
                      on={!kept.quiet.includes(channel)}
                      label={t(channel === "screen" ? "noticeScreen" : "noticeChime")}
                      disabled={held}
                      onPress={() =>
                        remember(
                          {
                            ...kept,
                            quiet: kept.quiet.includes(channel)
                              ? kept.quiet.filter((one) => one !== channel)
                              : [...kept.quiet, channel],
                          },
                          "notices",
                        )
                      }
                    />
                  </Line>
                ))}

              <Line
                title={t("updates")}
                why={t("lookNowWhen")}
                which="updates"
                said={said}
                trouble={trouble}
                more={
                  <>
                    {found === "none" && (
                      <p className="mt-1.5 text-[12px] text-soft">{t("lookNowNone")}</p>
                    )}
                    {found !== null && found !== "none" && (
                      <p className="mt-1.5 flex flex-wrap items-center gap-2 text-[12px] text-soft">
                        {fill("lookNowFound", found.version)}
                        {found.installs ? (
                          <button
                            type="button"
                            disabled={asked}
                            onClick={() => {
                              setAsked(true);
                              updateInstall().catch((e) => {
                                setAsked(false);
                                setTrouble({ card: "updates", text: saidPlainly(e) });
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
                  </>
                }
              >
                <button
                  type="button"
                  disabled={held || looking}
                  onClick={() => {
                    setLooking(true);
                    setFound(null);
                    updateReady(true)
                      .then((ready) => setFound(ready ?? "none"))
                      .catch((e) => setTrouble({ card: "updates", text: saidPlainly(e) }))
                      .finally(() => setLooking(false));
                  }}
                  className={mild}
                >
                  {looking ? t("lookingNow") : t("lookNow")}
                </button>
              </Line>
            </div>
            <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("noticesMore")}</p>

            <Band label={t("bandOutside")} />
            <div className="border-t border-hair">
              {reach?.shipped && (
                <Line
                  title={t("terminal")}
                  why={
                    reach.withinReach
                      ? fill("terminalOn", reach.through ?? reach.at ?? "")
                      : t("terminalOff")
                  }
                  which="terminal"
                  said={said}
                  trouble={trouble}
                  more={
                    reach.withinReach &&
                    !reach.onPath && (
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
                    )
                  }
                >
                  <Knob
                    on={reach.withinReach}
                    label={t(reach.withinReach ? "terminalRemove" : "terminalAdd")}
                    disabled={held}
                    onPress={() =>
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
                  />
                </Line>
              )}

              <Line
                title={t("greetAgain")}
                why={t("greetAgainWhy")}
                which="greet"
                said={said}
                trouble={trouble}
              >
                <button type="button" onClick={onGreet} className={mild}>
                  {t("greetAgainDo")}
                </button>
                <button
                  type="button"
                  disabled={held}
                  onClick={() =>
                    run("greet", guide(), (paper) => {
                      onChanged();
                      onDoc(paper.id);
                    })
                  }
                  className={mild}
                >
                  {t("welcomeGuide")}
                </button>
              </Line>
            </div>
          </>
        )}

        {tab === "data" && (
          <>
            <Band label={t("syncing")} />
            <section className="rounded-[10px] border border-hair px-4 py-3.5">
              <p className="text-[12.5px] leading-relaxed text-soft">
                {state.chosen ? fill("syncOn", state.chosen) : t("syncOff")}
              </p>
              {state.chosen && state.keeper && (
                <Warned keeper={state.keeper} named={state.keptBy} />
              )}
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
              {trouble?.card === "sync" && (
                <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>
              )}
              {said?.card === "sync" && (
                <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>
              )}
            </section>

            <Band label={t("attachTitle")} />
            <div className="border-t border-hair">
              {kept && (
                <Line
                  title={t("attachRow")}
                  why={t("attachWhy")}
                  which="attach"
                  said={said}
                  trouble={trouble}
                >
                  <select
                    aria-label={t("attachUpTo")}
                    value={String(kept.attachUpTo)}
                    disabled={held}
                    onChange={(e) =>
                      remember({ ...kept, attachUpTo: Number(e.target.value) }, "attach")
                    }
                    className={`rounded-[7px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
                  >
                    {SIZES.map((bytes) => (
                      <option key={bytes} value={bytes}>
                        {weigh(bytes)}
                      </option>
                    ))}
                  </select>
                </Line>
              )}

              {kept && (
                <Line
                  title={t("holdsTitle")}
                  why={
                    kept.shares
                      ? fill("holdsWhy", weigh(kept.onlySharedAbove))
                      : t("holdsNeedsShared")
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
                          setTrouble({ card: "holds", text: saidPlainly(e) });
                        });
                      }
                    }}
                    className={`rounded-[7px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
                  >
                    <option value="everywhere">{t("holdsEverywhere")}</option>
                    <option value="mine">{t("holdsMine")}</option>
                    <option value="shared">{t("holdsShared")}</option>
                  </select>
                </Line>
              )}
            </div>
            <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("attachBig")}</p>

            {state.backsUp && (
              <>
                <Band label={t("backup")} />
                <div className="border-t border-hair">
                  <Line
                    title={t("backupSave")}
                    why={
                      <>
                        <span className="block">{t("backupWhat")}</span>
                        <span className="mt-0.5 block tabular-nums">
                          {[
                            holds,
                            fill("backupAbout", weigh(state.weight)),
                            state.backedUpAt ? stamped(state.backedUpAt) : t("backupNever"),
                          ].join(" · ")}
                        </span>
                      </>
                    }
                    which="backup"
                    said={said}
                    trouble={trouble}
                  >
                    <button type="button" disabled={held} onClick={makeBackup} className={mild}>
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
                    <button type="button" disabled={held} onClick={takeBackup} className={risky}>
                      {t("restoreFrom")}
                    </button>
                  </Line>
                </div>
              </>
            )}

            <Band label={t("whereItLives")} />
            <div className="border-t border-hair">
              <Line
                title={t("aboutStore")}
                why={
                  <>
                    <span className="block font-mono text-[11.5px] break-all">
                      {build?.store ?? "…"}
                    </span>
                    <span className="mt-0.5 block">{t("storeFixed")}</span>
                  </>
                }
                which="store"
                said={said}
                trouble={trouble}
              >
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
              </Line>
            </div>
          </>
        )}

        {tab === "agents" && (
          <>
            <Group label={t("tabAgents")} />

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

            <Card title={t("wiringTitle")} which="wiring" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("wiringWhat")}</p>

              {agents?.length === 0 && (
                <p className="mt-3 text-[12.5px] leading-relaxed text-faint">{t("wiringNone")}</p>
              )}

              {agents && agents.length > 0 && (
                <>
                  <div className="mt-3 overflow-hidden rounded-lg border border-hair">
                    {agents.map((one) => (
                      <div
                        key={one.id}
                        className="flex items-center gap-3 border-t border-hair px-3 py-2.5 first:border-t-0"
                      >
                        <span className="min-w-0 flex-1">
                          <span className="block text-[13px] font-semibold">{one.name}</span>
                          <span className="block truncate font-mono text-[10.5px] text-faint">
                            {one.at}
                          </span>
                          {one.astray && (
                            <span className="block text-[11.5px] text-high">
                              {t("wiringAstray")}
                            </span>
                          )}
                        </span>
                        {one.wired && !one.astray && (
                          <span className="shrink-0 rounded-full border border-hue-green/40 px-2 py-0.5 text-[11.5px] text-hue-green">
                            {t("wiringOn")}
                          </span>
                        )}
                        <button
                          type="button"
                          disabled={held}
                          onClick={() => join(one)}
                          className={`shrink-0 rounded-md border px-2.5 py-1 text-[12px] disabled:text-faint ${
                            one.wired && !one.astray
                              ? "border-line text-soft hover:border-urgent hover:text-urgent"
                              : "border-accent text-accent"
                          }`}
                        >
                          {one.astray
                            ? t("wiringAgain")
                            : one.wired
                              ? t("wiringOut")
                              : t("wiringJoin")}
                        </button>
                      </div>
                    ))}
                  </div>

                  {agent?.on === false && (
                    <p className="mt-2.5 text-[12.5px] leading-relaxed text-soft">
                      {t("wiringMute")}
                    </p>
                  )}
                  <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">
                    {t("wiringBefore")}
                  </p>
                </>
              )}
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

              <p className="mt-3 text-[12px] font-semibold">{t("agentsByFile")}</p>
              <pre className="mt-1 overflow-x-auto rounded-lg border border-hair px-3 py-2 font-mono text-[11.5px] text-soft">
                {wiring(reach?.binary)}
              </pre>
              <button
                type="button"
                onClick={() => {
                  void copied(wiring(reach?.binary)).then(() => {
                    setWired(true);
                    window.setTimeout(() => setWired(false), 1500);
                  });
                }}
                className="mt-2 rounded-md border border-line px-2.5 py-0.5 text-[12px] text-soft hover:border-accent hover:text-accent"
              >
                {wired ? t("agentsCopied") : t("agentsCopy")}
              </button>

              <p className="mt-4 text-[12px] font-semibold">{t("agentsByLine")}</p>
              <pre className="mt-1 overflow-x-auto rounded-lg border border-hair px-3 py-2 font-mono text-[11.5px] text-soft">
                {oneLine(reach?.binary, t("agentsCalled"))}
              </pre>
              <p className="mt-1 text-[11.5px] text-faint">{t("agentsWhichever")}</p>
              <button
                type="button"
                onClick={() => {
                  void copied(oneLine(reach?.binary, t("agentsCalled"))).then(() => {
                    setTyped(true);
                    window.setTimeout(() => setTyped(false), 1500);
                  });
                }}
                className="mt-2 rounded-md border border-line px-2.5 py-0.5 text-[12px] text-soft hover:border-accent hover:text-accent"
              >
                {typed ? t("agentsCopied") : t("agentsCopy")}
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
                            {`${weigh(one.bytes)} · ${dated(one.when)}${
                              one.shared ? ` · ${t("looseShared")}` : ""
                            }`}
                          </span>
                          <button
                            type="button"
                            disabled={held}
                            onClick={() => letGo(one.at, one.shared)}
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
                      disabled={held || audit.loose === 0}
                      onClick={() => letGoOfAll(audit.astray)}
                      className="rounded-[7px] border border-urgent/45 px-2.5 py-1 text-[12.5px] text-urgent hover:bg-hover disabled:border-hair disabled:text-faint"
                    >
                      {t("upkeepSafeAll")}
                    </button>
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

            <Group label={t("upkeepLook")} />

            <Card title={t("upkeepLook")} which="review" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("upkeepLookWhat")}</p>
              {audit && audit.stranded.length === 0 && (
                <p className="mt-2 text-[12.5px] text-faint">{t("upkeepNothing")}</p>
              )}
              {audit && audit.stranded.length > 0 && (
                <>
                  <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
                    {audit.stranded.map((one) => (
                      <li key={one.file} className="flex items-baseline justify-between gap-4">
                        <span className="min-w-0">
                          <span className="block truncate">{one.title || t("untitledDoc")}</span>
                          <span className="block font-mono text-[10.5px] text-faint">
                            {`${one.file} · ${weigh(one.bytes)} · ${dated(one.when)}`}
                          </span>
                        </span>
                        <span className="flex shrink-0 items-baseline gap-2.5">
                          <button
                            type="button"
                            disabled={held}
                            onClick={() => takeIn(one.file)}
                            className="text-[11.5px] text-accent hover:underline disabled:text-soft"
                          >
                            {t("upkeepTakeIn")}
                          </button>
                          <button
                            type="button"
                            disabled={held}
                            onClick={() => letGoOfPaper(one)}
                            className="text-[11.5px] text-urgent hover:underline disabled:text-soft"
                          >
                            {t("upkeepDropIt")}
                          </button>
                        </span>
                      </li>
                    ))}
                  </ul>
                  <div className="mt-2.5">
                    <button
                      type="button"
                      disabled={held}
                      onClick={() => takeInAll(audit.stranded)}
                      className="rounded-[7px] border border-line px-2.5 py-1 text-[12.5px] hover:bg-hover disabled:border-hair disabled:text-faint"
                    >
                      {t("upkeepTakeInAll")}
                    </button>
                  </div>
                </>
              )}
            </Card>

            <Group label={fill("upkeepWaiting", hushedName(audit) ?? t("theMachines"))} />

            <Card
              title={fill("upkeepWaiting", hushedName(audit) ?? t("theMachines"))}
              which="review"
              busy={busy}
              said={said}
              trouble={trouble}
            >
              <p className="text-[12.5px] leading-relaxed text-soft">{t("upkeepWaitingWhat")}</p>
              {audit && audit.missing.length === 0 && (
                <p className="mt-2 text-[12.5px] text-faint">{t("upkeepNothing")}</p>
              )}
              {audit && audit.missing.length > 0 && (
                <ul className="scroller mt-2 flex max-h-[22rem] flex-col gap-1 overflow-y-auto text-[12.5px]">
                  {audit.missing.map((one) => (
                    <li key={one.file} className="flex items-baseline justify-between gap-4">
                      <span className="min-w-0">
                        <span className="block truncate">{one.title || t("untitledDoc")}</span>
                        <span className="block font-mono text-[10.5px] text-faint">{one.file}</span>
                      </span>
                      <span className="flex shrink-0 items-baseline gap-2.5">
                        {hushedName(audit) ? (
                          <span className="text-[11.5px] text-faint">
                            {fill("upkeepForgetWaits", hushedName(audit) ?? "")}
                          </span>
                        ) : (
                          <button
                            type="button"
                            disabled={held}
                            onClick={() => forgetMissing(one)}
                            className="text-[11.5px] text-urgent hover:underline disabled:text-soft"
                          >
                            {t("upkeepForget")}
                          </button>
                        )}
                      </span>
                    </li>
                  ))}
                </ul>
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

const hushedName = (audit: Reviewed | null): string | null =>
  audit?.machines.find(hushed)?.called ?? null;

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

const wiring = (at?: string) =>
  `{
  "mcpServers": {
    "tisty": { "command": ${JSON.stringify(at ?? "tisty")}, "args": ["mcp"] }
  }
}`;

const oneLine = (at?: string, agent = "agent") =>
  `${agent} mcp add tisty -- ${JSON.stringify(at ?? "tisty")} mcp`;

function Band({ label }: { label: string }) {
  return (
    <div className="mt-5 mb-1.5 text-[11px] font-semibold tracking-[0.06em] text-faint uppercase">
      {label}
    </div>
  );
}

function Line({
  title,
  why,
  which,
  said,
  trouble,
  children,
  more,
}: {
  title: string;
  why?: React.ReactNode;
  which: Which;
  said?: Word;
  trouble?: Word;
  children?: React.ReactNode;
  more?: React.ReactNode;
}) {
  return (
    <div className="border-b border-hair py-2.5">
      <div className="flex items-center gap-4">
        <span className="min-w-0 flex-1">
          <span className="block text-[13px] font-medium">{title}</span>
          {why && <span className="mt-px block text-[12px] leading-snug text-faint">{why}</span>}
        </span>
        {children && (
          <span className="flex shrink-0 flex-wrap items-center justify-end gap-2">{children}</span>
        )}
      </div>
      {more}
      {trouble?.card === which && (
        <p className="mt-1.5 text-[11.5px] text-urgent">{trouble.text}</p>
      )}
      {said?.card === which && <p className="mt-1.5 text-[11.5px] text-faint">{said.text}</p>}
    </div>
  );
}

function Knob({
  on,
  label,
  disabled,
  onPress,
}: {
  on: boolean;
  label: string;
  disabled?: boolean;
  onPress: () => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      disabled={disabled}
      onClick={onPress}
      className={`relative h-5 w-[34px] shrink-0 rounded-full transition-colors motion-reduce:transition-none disabled:opacity-50 ${
        on ? "bg-accent" : "bg-hair"
      }`}
    >
      <span
        className={`absolute top-0.5 block size-4 rounded-full bg-bg shadow-sm transition-[left] motion-reduce:transition-none ${
          on ? "left-[16px]" : "left-0.5"
        }`}
      />
    </button>
  );
}

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
  restore: "restoreTitle",
  review: "review",
  brittle: "brittleAre",
  terminal: "terminal",
  quick: "quick",
  waking: "wake",
  greet: "greetAgain",
  tongue: "tongue",
  settings: "settingsTitle",
  notices: "bandNotices",
  updates: "updates",
  attach: "attachTitle",
  holds: "holdsTitle",
  wiring: "wiringTitle",
  report: "reportTitle",
  store: "aboutStore",
};

const TAIL = 300;
const LOGS = "\n--- tisty.log ---";

function Warned({ keeper, named }: { keeper: Keeper; named?: string }) {
  const warning = warningOf(keeper, named);
  return (
    <div
      className={`mt-2 rounded-lg px-3 py-2 text-[12px] leading-relaxed text-soft ${
        warning.mild ? "bg-accent-soft" : "border border-hue-amber/40"
      }`}
    >
      <span className="block text-[12.5px] font-semibold text-ink">{warning.said}</span>
      {warning.why}
    </div>
  );
}

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
