import { listen } from "@tauri-apps/api/event";
import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
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
import Sealing from "./Sealing";
import { SECTIONS } from "./SettingsNav";
import { useSigning } from "./Signing";
import Sync from "./Sync";
import Upkeep, { type Seen } from "./Upkeep";
import { useCards } from "./useCards";
import { useCarry } from "./useCarry";
import Yours from "./Yours";

interface Props {
  onPack: () => void;
  onUnpack: () => void;
  onChanged: () => void;
  onGreet: () => void;
  onDoc: (paper: string) => void;
  greeted?: number;
  tab: Tab;
  onTab: (tab: Tab) => void;
  move?: Heading;
  movedAt?: number;
  about?: ReactNode;
}

export default function Keeping({
  onPack,
  onUnpack,
  onChanged,
  onGreet,
  onDoc,
  greeted,
  tab,
  onTab,
  move,
  movedAt,
  about,
}: Props) {
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
    if (!move) return;
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

  if (tab === "about" && about) {
    return (
      <>
        {carrying.shown}
        {about}
      </>
    );
  }

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
          <h2 className="mb-3.5 text-[21px] font-semibold">
            {t(SECTIONS.find((one) => one.key === tab)?.label ?? "keeping")}
          </h2>

          {busy !== null &&
            (tab === "general" || tab === "sync" || tab === "security" || tab === "data") && (
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

          {tab === "security" && (
            <Identity
              round={state.last}
              busy={busy}
              said={said}
              trouble={trouble}
              tell={setSaid}
              fail={setTrouble}
              onSync={() => onTab("sync")}
            />
          )}
          {tab === "security" && (
            <Sealing
              busy={busy}
              said={said}
              trouble={trouble}
              tell={setSaid}
              fail={setTrouble}
              onChanged={onChanged}
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
              onSync={() => onTab("sync")}
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
