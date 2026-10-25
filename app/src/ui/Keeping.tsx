import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  type About,
  about,
  chooseSync,
  cloudForget,
  cloudStatus,
  type Freeing,
  keepSettings,
  type Reach,
  type Reviewed,
  reachable,
  settings as readSettings,
  type Settings,
  shortcut,
  type Toward,
  type Waking,
  waking,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { afterMove } from "../relocation";
import { refresh, useSyncSeen } from "../syncSeen";
import type { Heading, Tab } from "../views";
import Agents, { useAgents } from "./Agents";
import { NAMED, type Which } from "./Card";
import Choosing from "./Choosing";
import General from "./General";
import Identity from "./Identity";
import Modal from "./Modal";
import Relocating from "./Relocating";
import { strong } from "./Rows";
import { useSigning } from "./Signing";
import Sync from "./Sync";
import Upkeep, { type Seen } from "./Upkeep";
import { useCards } from "./useCards";
import { useCarry } from "./useCarry";
import Yours from "./Yours";

const TABS: { key: Tab; label: Parameters<typeof t>[0] }[] = [
  { key: "general", label: "tabGeneral" },
  { key: "sync", label: "tabSync" },
  { key: "identity", label: "tabIdentity" },
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
  move?: Heading;
  movedAt?: number;
}

export default function Keeping({
  onPack,
  onUnpack,
  onChanged,
  onGreet,
  onDoc,
  greeted,
  start,
  move,
  movedAt,
}: Props) {
  const [tab, setTab] = useState<Tab>(start ?? "general");
  const shared = useSyncSeen();
  const [read, setRead] = useState(false);
  // What another view read before this one opened may be old, so the settings wait for their own look.
  const state = read ? shared.state : null;
  const [audit, setAudit] = useState<Reviewed | null>(null);
  const [kept, setKept] = useState<Settings | null>(null);
  const [freeing, setFreeing] = useFreeing();
  const [freeDays, setFreeDays] = useState(90);
  const [seen, setSeen] = useState<Seen | null>(null);
  const { busy, busySince, setBusy, said, setSaid, trouble, setTrouble, run, quietly } = useCards(
    () => {
      look();
      onChanged();
    },
  );
  const signs = useSigning();
  const { lookSigned } = signs;
  const known = useAgents(tab === "agents", setTrouble);

  const look = useCallback(() => {
    lookSigned();
    refresh()
      .then(() => setRead(true))
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
    readSettings()
      .then(setKept)
      .catch(() => {});
  }, [lookSigned, setTrouble]);

  useEffect(look, [look]);
  const { reach, wake, setWake, keys, setKeys, build } = useAbout(greeted, look);

  const [picking, setPicking] = useState(false);
  const [moving, setMoving] = useState<{ heading?: Heading; resume?: Toward } | null>(null);

  useEffect(() => {
    if (start) setTab(start);
  }, [start, movedAt]);

  useEffect(() => {
    if (!move) return;
    setTab("sync");
    setMoving({ heading: move });
  }, [move, movedAt]);
  const [was, setWas] = useState<string>();
  const held = busy !== null;

  const carrying = useCarry({
    held,
    cloud: state?.mode === "cloud" ? t("cloudChosen") : undefined,
    setBusy,
    tell: setSaid,
    fail: setTrouble,
    look,
    onChanged,
  });
  const carryNow = carrying.carryNow;
  const landed = useRef(() => {});
  landed.current = () => {
    look();
    void carryNow();
  };
  useEffect(() => afterMove(() => landed.current()), []);

  if (!state) {
    return (
      <Unread
        trouble={trouble?.text}
        retry={() => {
          setTrouble(undefined);
          look();
        }}
      />
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
    const brought = at === "cloud" ? await cloudStatus().catch(() => null) : undefined;
    await chooseSync(was)
      .then(() => (brought ? cloudForget(brought.account, brought.provider) : undefined))
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
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
          <Choosing
            busy={held}
            offered={state.cloudOffered && state.mode !== "folder"}
            onTrouble={(text) => setTrouble(text ? { card: "sync", text } : undefined)}
            onDone={picked}
          />
          {trouble?.card === "sync" && (
            <p role="alert" className="mt-3 text-[11.5px] text-urgent">
              {trouble.text}
            </p>
          )}
        </Modal>
      )}
      {moving && (
        <Relocating
          state={state}
          heading={moving.heading}
          resume={moving.resume}
          onClose={() => {
            setMoving(null);
            look();
          }}
        />
      )}
      {carrying.shown}
      <div data-tauri-drag-region className="h-9 shrink-0" />
      {/* The whole width scrolls, so the wheel moves the page wherever the pointer is. */}
      <div className="scroller min-h-0 flex-1">
        <div className="mx-auto w-full max-w-[560px] px-6 pb-12">
          <h2 className="mb-3.5 text-[21px] font-semibold">{t("keeping")}</h2>

          <Tabs tab={tab} setTab={setTab} />

          {busy !== null && (tab === "general" || tab === "sync" || tab === "data") && (
            <p className="text-[11.5px] text-faint">{fill("waitFor", t(NAMED[busy]))}</p>
          )}

          {tab === "general" && (
            <General
              kept={kept}
              setKept={setKept}
              wake={wake}
              setWake={setWake}
              keys={keys}
              setKeys={setKeys}
              busy={busy}
              said={said}
              trouble={trouble}
              run={run}
              tell={setSaid}
              remember={remember}
              signs={signs}
              onChanged={onChanged}
              onGreet={onGreet}
              onDoc={onDoc}
            />
          )}

          {tab === "sync" && (
            <Sync
              state={state}
              kept={kept}
              audit={audit}
              setAudit={setAudit}
              freeing={freeing}
              setFreeing={setFreeing}
              busy={busy}
              busySince={busySince}
              said={said}
              trouble={trouble}
              run={run}
              tell={setSaid}
              fail={setTrouble}
              carry={(way) => void carryNow(way)}
              pickFolder={pickFolder}
              onMove={(resume) => setMoving({ resume })}
              remember={remember}
            />
          )}

          {tab === "identity" && (
            <Identity
              audit={audit}
              setAudit={setAudit}
              busy={busy}
              said={said}
              trouble={trouble}
              tell={setSaid}
              fail={setTrouble}
              onSync={() => setTab("sync")}
            />
          )}

          {tab === "data" && (
            <Yours
              state={state}
              holds={holds}
              build={build}
              busy={busy}
              said={said}
              trouble={trouble}
              run={run}
              fail={setTrouble}
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
              known={known}
            />
          )}

          {tab === "upkeep" && (
            <Upkeep
              audit={audit}
              setAudit={setAudit}
              build={build}
              state={state}
              freeing={freeing}
              setFreeing={setFreeing}
              freeDays={freeDays}
              setFreeDays={setFreeDays}
              busy={busy}
              busySince={busySince}
              said={said}
              trouble={trouble}
              run={run}
              quietly={quietly}
              tell={setSaid}
              fail={setTrouble}
              onSync={() => setTab("sync")}
              onDoc={onDoc}
              seen={seen}
              setSeen={setSeen}
            />
          )}
        </div>
      </div>
    </main>
  );
}

function Tabs({ tab, setTab }: { tab: Tab; setTab: (tab: Tab) => void }) {
  return (
    <div role="tablist" className="mb-4 flex flex-wrap gap-1">
      {TABS.map((one) => (
        <button
          key={one.key}
          type="button"
          role="tab"
          aria-selected={tab === one.key}
          onClick={() => setTab(one.key)}
          className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
            tab === one.key ? "border-ink bg-ink text-bg" : "border-line text-faint hover:text-soft"
          }`}
        >
          {t(one.label)}
        </button>
      ))}
    </div>
  );
}

function Unread({ trouble, retry }: { trouble?: string; retry: () => void }) {
  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="mx-auto w-full max-w-[560px] px-6">
        <h2 className="mb-3.5 text-[21px] font-semibold">{t("keeping")}</h2>
        {trouble && (
          <div className="rounded-[10px] border border-hair bg-panel p-4">
            <p role="alert" className="text-[12.5px] leading-relaxed text-urgent">
              {trouble}
            </p>
            <button type="button" onClick={retry} className={`mt-2.5 ${strong}`}>
              {t("tryAgain")}
            </button>
          </div>
        )}
      </div>
    </main>
  );
}

function useFreeing() {
  const [freeing, setFreeing] = useState<Freeing | null>(null);
  useEffect(() => {
    const off = listen<Freeing>("freeing", (told) => setFreeing(told.payload));
    return () => {
      void off.then((stop) => stop());
    };
  }, []);
  return [freeing, setFreeing] as const;
}

function useAbout(greeted: number | undefined, look: () => void) {
  const [reach, setReach] = useState<Reach | null>(null);
  const [wake, setWake] = useState<Waking | null>(null);
  const [keys, setKeys] = useState<string | null>(null);
  const [build, setBuild] = useState<About | null>(null);
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
  return { reach, wake, setWake, keys, setKeys, build };
}
