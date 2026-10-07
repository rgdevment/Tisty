import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import {
  type About,
  about,
  type Carrying,
  chooseSync,
  type Freeing,
  keepSettings,
  type Reach,
  type Reviewed,
  reachable,
  settings as readSettings,
  type Settings,
  shortcut,
  syncState,
  type Waking,
  waking,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import type { Tab } from "../views";
import Agents from "./Agents";
import { NAMED, type Which, type Word } from "./Card";
import General from "./General";
import Keepers from "./Keepers";
import Modal from "./Modal";
import { strong } from "./Rows";
import Upkeep from "./Upkeep";
import { useCarry } from "./useCarry";
import Yours from "./Yours";

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
  const held = busy !== null;

  const carrying = useCarry({
    held,
    setBusy,
    tell: setSaid,
    fail: setTrouble,
    look,
    onChanged,
  });
  const carryNow = carrying.carryNow;

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
      {carrying.shown}
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
          <Yours
            state={state}
            kept={kept}
            holds={holds}
            build={build}
            freeing={freeing}
            setFreeing={setFreeing}
            busy={busy}
            said={said}
            trouble={trouble}
            run={run}
            fail={setTrouble}
            carry={(way) => void carryNow(way)}
            pickFolder={pickFolder}
            remember={remember}
            onPack={onPack}
            onUnpack={onUnpack}
            tell={setSaid}
          />
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
          <Upkeep
            audit={audit}
            setAudit={setAudit}
            build={build}
            busy={busy}
            said={said}
            trouble={trouble}
            run={run}
            quietly={quietly}
            tell={setSaid}
            fail={setTrouble}
          />
        )}
      </div>
    </main>
  );
}
