import { listen } from "@tauri-apps/api/event";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { stillApart, walkThrough } from "../apart";
import {
  type About,
  type Astray,
  about,
  backUp,
  type Carrying,
  checked,
  chooseSync,
  confirmMachineKey,
  docAdopt,
  docDrop,
  docLetGo,
  docs,
  type Freeing,
  freeUp,
  type Gone,
  type Holds,
  type Kin,
  keepSettings,
  type Machine,
  type Reach,
  type Reviewed,
  reachable,
  settings as readSettings,
  readTags,
  rebuild,
  removeMachine,
  restore,
  retireAttachment,
  retireAttachments,
  revealed,
  type Settings,
  type Stray,
  shortcut,
  stopFreeing,
  syncKin,
  syncNow,
  syncState,
  type Waking,
  waking,
  whatWentAmiss,
} from "../core";
import { decideAll } from "../deciding";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { syncSaid } from "../syncSaid";
import type { Tab } from "../views";
import Agents from "./Agents";
import Apart, { type Door } from "./Apart";
import Backup from "./Backup";
import Card, { NAMED, type Which, type Word } from "./Card";
import General from "./General";
import Keepers from "./Keepers";
import { Asked, MachineList } from "./Keys";
import Leftovers from "./Leftovers";
import Modal from "./Modal";
import Reporting from "./Reporting";
import { Band, dated, Group, Line, mild, off, strong } from "./Rows";
import Syncing from "./Syncing";
import Tidying from "./Tidying";

const TABS: { key: Tab; label: Parameters<typeof t>[0] }[] = [
  { key: "general", label: "tabGeneral" },
  { key: "data", label: "tabData" },
  { key: "agents", label: "tabAgents" },
  { key: "upkeep", label: "tabUpkeep" },
];

interface Props {
  onPack: () => void;
  onUnpack: () => void;
  onChanged: () => void;
  onGreet: () => void;
  onDoc: (paper: string) => void;
  greeted?: number;
  start?: Tab;
}

export default function Keeping({
  onPack,
  onUnpack,
  onChanged,
  onGreet,
  onDoc,
  greeted,
  start,
}: Props) {
  const [tab, setTab] = useState<Tab>(start ?? "general");
  const [state, setState] = useState<Carrying | null>(null);
  const [audit, setAudit] = useState<Reviewed | null>(null);
  const [keyOf, setKeyOf] = useState<Machine | null>(null);
  const [astray, setAstray] = useState<Machine | null>(null);
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

  const look = useCallback(() => {
    syncState()
      .then(setState)
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
    readSettings()
      .then(setKept)
      .catch(() => {});
  }, []);

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
            <div className="rounded-[10px] border border-hair bg-panel p-4">
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
        if (!stillApart(problem)) throw problem;
        setKin(await syncKin().catch(() => "unsure" as const));
        const door = await new Promise<Door | "else" | null>((settle) => setApart(() => settle));
        if (door === null) return "declined" as const;
        if (!(await walkThrough(door))) return "declined" as const;
        return syncNow();
      });

      if (answer === "declined") {
        setTrouble({ card: "sync", text: t("wouldReset") });
        return "declined";
      }
      const shut = await decideAll(answer.undecided);
      const amiss = whatWentAmiss(answer);
      if (amiss) {
        setTrouble({ card: "sync", text: t(amiss) });
      } else if (shut.length) {
        setTrouble({ card: "sync", text: fill("someLockedAtOdds", await namedDocs(shut)) });
      } else if (answer.joined?.length) {
        setSaid({ card: "sync", text: fill("someJoined", await namedDocs(answer.joined)) });
      } else {
        setSaid({ card: "sync", text: syncSaid(answer) });
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
      .then((sure) => {
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

  // A refusal says "look again", so the window is the one that looks: leaving the stale key and a
  // live button on the row is how somebody clicks the same refusal forever.
  const confirmKey = (one: Machine) => {
    if (held || !one.signs) return;
    const said = one.signs;
    setKeyOf(null);
    run(
      "machines",
      confirmMachineKey(one.id, said)
        .then(() => null)
        .catch((e) => saidPlainly(e))
        .then((amiss) => checked().then((now) => ({ amiss, now }))),
      ({ amiss, now }) => {
        setAudit(now);
        if (amiss) setTrouble({ card: "machines", text: amiss });
        else setSaid({ card: "machines", text: t("machineKeyDone") });
      },
    );
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
          run("machines", removeMachine(one.id).then(checked), (now) => {
            setAudit(now);
            setSaid({ card: "machines", text: t("machineDropped") });
          }),
      )
      .catch((e) => setTrouble({ card: "machines", text: saidPlainly(e) }));
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

  const holds = [
    fill("openTasks", String(state.open)),
    fill("archivedTasks", String(state.archived)),
    fill("reviewLists", String(state.lists)),
    fill("someAttachments", String(state.attachments)),
  ].join(" · ");

  return (
    <main className="flex flex-col overflow-hidden">
      <Asked
        keyOf={keyOf}
        astray={astray}
        busy={held}
        onConfirm={confirmKey}
        onClose={() => {
          setKeyOf(null);
          setAstray(null);
        }}
      />
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
          <General
            kept={kept}
            setKept={setKept}
            wake={wake}
            setWake={setWake}
            keys={keys}
            reach={reach}
            setReach={setReach}
            busy={busy}
            said={said}
            trouble={trouble}
            run={run}
            tell={setSaid}
            remember={remember}
            greeted={greeted}
            onChanged={onChanged}
            onGreet={onGreet}
            onDoc={onDoc}
          />
        )}

        {tab === "data" && (
          <>
            <Syncing
              state={state}
              kept={kept}
              busy={busy}
              said={said}
              trouble={trouble}
              run={run}
              fail={setTrouble}
              carry={(way) => void carryNow(way)}
              pickFolder={pickFolder}
            />

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
                    className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
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
                      if (holds === "everywhere") {
                        void carryNow();
                      }
                    }}
                    className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
                  >
                    <option value="everywhere">{t("holdsEverywhere")}</option>
                    <option value="mine">{t("holdsMine")}</option>
                    <option value="shared">{t("holdsShared")}</option>
                  </select>
                </Line>
              )}
            </div>
            <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("attachBig")}</p>

            <Band label={t("bandParcels")} />
            <div className="border-t border-hair">
              <Line
                title={t("packAllPlain")}
                why={t("packAllWhy")}
                which="parcel"
                said={said}
                trouble={trouble}
              >
                <button
                  type="button"
                  disabled={held}
                  onClick={onPack}
                  className={`rounded-[10px] border border-line px-2 py-1 text-[12.5px] hover:bg-hover ${off}`}
                >
                  {t("packAllDo")}
                </button>
              </Line>
              <Line
                title={t("unpackPlain")}
                why={t("unpackWhy")}
                which="parcel"
                said={said}
                trouble={trouble}
              >
                <button
                  type="button"
                  disabled={held}
                  onClick={onUnpack}
                  className={`rounded-[10px] border border-line px-2 py-1 text-[12.5px] hover:bg-hover ${off}`}
                >
                  {t("unpackDo")}
                </button>
              </Line>
            </div>

            <Backup
              state={state}
              holds={holds}
              held={held}
              said={said}
              trouble={trouble}
              onMake={makeBackup}
              onTake={takeBackup}
            />

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
          <Agents
            busy={busy}
            said={said}
            trouble={trouble}
            quietly={quietly}
            tell={setSaid}
            fail={setTrouble}
            binary={reach?.binary}
          />
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

            <Card
              title={t("theMachines")}
              which="machines"
              busy={busy}
              said={said}
              trouble={trouble}
            >
              <p className="text-[12.5px] leading-relaxed text-soft">{t("machinesWhat")}</p>
              <MachineList
                all={audit?.machines ?? null}
                busy={held}
                onKey={setKeyOf}
                onAstray={setAstray}
                onDrop={dropMachine}
              />
            </Card>

            <Card title={t("tagsRead")} which="tagging" busy={busy} said={said} trouble={trouble}>
              <p className="text-[12.5px] leading-relaxed text-soft">{t("tagsReadWhy")}</p>
              <div className="mt-2.5 flex items-center gap-2.5">
                <button
                  type="button"
                  disabled={held}
                  onClick={() =>
                    run("tagging", readTags(), (many) =>
                      setSaid({ card: "tagging", text: fill("tagsReadDone", String(many)) }),
                    )
                  }
                  className={mild}
                >
                  {t("tagsReadDo")}
                </button>
              </div>
            </Card>

            <Tidying
              busy={busy}
              said={said}
              trouble={trouble}
              run={run}
              tell={setSaid}
              mild={mild}
            />

            <Leftovers
              audit={audit}
              build={build}
              busy={busy}
              said={said}
              trouble={trouble}
              run={run}
              fail={setTrouble}
              letGo={letGo}
              letGoOfAll={letGoOfAll}
              takeIn={takeIn}
              takeInAll={takeInAll}
              letGoOfPaper={letGoOfPaper}
              forgetMissing={forgetMissing}
            />

            <Reporting
              busy={busy}
              said={said}
              trouble={trouble}
              tell={setSaid}
              fail={setTrouble}
              quietly={quietly}
            />
          </>
        )}
      </div>
    </main>
  );
}

const SIZES = [256 * 1024, 1024 * 1024, 5 * 1024 * 1024, 20 * 1024 * 1024, 50 * 1024 * 1024];
