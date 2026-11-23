import { listen } from "@tauri-apps/api/event";
import {
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import {
  type About,
  about,
  chooseSync,
  cloudForget,
  cloudStatus,
  type Freeing,
  type Reach,
  type Ready,
  type Reviewed,
  reachable,
  settings as readSettings,
  type Settings,
  shortcut,
  type Toward,
  type Waking,
  waking,
} from "../core";
import { useKnownAgents } from "../knownAgents";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { afterMove } from "../relocation";
import type { SettingsDesk } from "../settingsDesk";
import { type Article, pageOf } from "../settingsPages";
import type { RowId, Shelf, Spot } from "../settingsRows";
import { refresh, useSyncSeen } from "../syncSeen";
import type { Heading, Tab } from "../views";
import { useAgentsLook } from "./Agents";
import Choosing from "./Choosing";
import Found from "./Found";
import KeepingPage from "./KeepingPages";
import { useLanding } from "./landing";
import Modal from "./Modal";
import Relocating from "./Relocating";
import { HeedContext, RowsContext } from "./RowNotes";
import { strong } from "./Rows";
import { useRemember } from "./remembering";
import { useSigning } from "./Signing";
import Summary from "./Summary";
import { PANEL } from "./settingsPanel";
import type { Seen } from "./Upkeep";
import { useCards } from "./useCards";
import { useCarry } from "./useCarry";
import { useExtras } from "./useExtras";
import { useOverview } from "./useOverview";
import { held } from "./working";
import { PageHead } from "./Zone";

const KEPT = 1;
const KEYS = 2;
const BUILD = 4;
const ALL_READ = KEPT | KEYS | BUILD;

interface Props {
  onPack: () => void;
  onUnpack: () => void;
  onChanged: () => void;
  onDoc: (paper: string) => void;
  greeted?: number;
  tab: Tab;
  onTab: (tab: Tab) => void;
  move?: Heading;
  movedAt?: number;
  about?: ReactNode;
  hidden?: ReactNode;
  onHelp?: (article?: Article) => void;
  ready?: Ready | null;
  desk?: SettingsDesk;
  onLand?: (spot: Spot) => void;
  shelf?: Shelf;
  row?: RowId;
  landedAt?: number;
}

export default function Keeping({
  onPack,
  onUnpack,
  onChanged,
  onDoc,
  greeted,
  tab,
  onTab,
  move,
  movedAt,
  about,
  hidden,
  onHelp,
  ready = null,
  desk,
  onLand,
  shelf,
  row,
  landedAt,
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
  const cards = useCards(() => {
    look();
    onChanged();
  });
  const { busy, said, trouble, run, quietly, tell, fail } = cards;
  const setTrouble = fail;
  const signs = useSigning();
  const { lookSigned } = signs;
  const own = useKnownAgents(Boolean(desk));
  const known = desk?.agents ?? own;
  useAgentsLook(known, tab === "agents", setTrouble);
  const offering = useExtras(known);
  const [answered, answer] = useReducer((was: number, read: number) => was | read, 0);

  const look = useCallback(() => {
    lookSigned();
    refresh()
      .then(() => setRead(true))
      .catch((e) => setTrouble({ card: "sync", text: saidPlainly(e) }));
    readSettings()
      .then(setKept)
      .catch(() => {})
      .finally(() => answer(KEPT));
  }, [lookSigned, setTrouble]);
  const reread = useCallback(() => {
    readSettings()
      .then(setKept)
      .catch(() => {});
  }, []);
  const remember = useRemember({ setKept, quietly, tell, reread, onChanged });
  const amend = useCallback(
    (edit: (was: Settings) => Settings) => setKept((was) => (was ? edit(was) : was)),
    [],
  );

  useEffect(look, [look]);
  const { reach, wake, setWake, keys, keysKnown, setKeys, build, lookAbout } = useAbout(
    greeted,
    look,
    answer,
  );
  const overview = useOverview({
    state,
    status: shared.status,
    build,
    kept,
    keys: keysKnown ? keys : undefined,
    alias: signs.signedAlias,
    ready,
    audit,
    seen,
    wired: known.agents,
    skills: known.skills,
    waiting: answered !== ALL_READ,
    desk,
  });
  useLanding(row, landedAt);
  // What another page changed since opening, a phrase made or a door turned, is read again on coming back.
  const summarized = useRef(false);
  useEffect(() => {
    if (tab !== "summary") return;
    if (summarized.current) lookAbout();
    summarized.current = true;
  }, [tab, lookAbout]);
  const land = (spot: Spot) => {
    desk?.setFinding("");
    if (onLand) onLand(spot);
    else onTab(spot.page);
  };
  const finding = desk?.finding.trim() ?? "";

  const [picking, setPicking] = useState(false);
  const [moving, setMoving] = useState<{ heading?: Heading; resume?: Toward } | null>(null);

  useEffect(() => {
    if (!move) return;
    setMoving({ heading: move });
  }, [move, movedAt]);
  const [was, setWas] = useState<string>();
  const carrying = useCarry({
    held: held(busy, "sync"),
    cloud: state?.mode === "cloud" ? t("cloudChosen") : undefined,
    start: cards.start,
    stop: cards.stop,
    clear: cards.clear,
    tell,
    fail,
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

  const rows = useMemo(() => ({ busy, said, trouble }), [busy, said, trouble]);
  const shownItems = overview.items ?? desk?.attention ?? null;

  if (tab === "about" && about && !finding) {
    return (
      <RowsContext.Provider value={rows}>
        <HeedContext.Provider value={shownItems}>
          {carrying.shown}
          <div id={PANEL} role="tabpanel" aria-label={t(pageOf(tab).label)} className="contents">
            {about}
          </div>
        </HeedContext.Provider>
      </RowsContext.Provider>
    );
  }

  if (!state) {
    return (
      <Unread
        tab={tab}
        trouble={trouble.sync?.text}
        retry={() => {
          cards.clear("sync");
          look();
        }}
      />
    );
  }

  const pickFolder = () => {
    if (held(busy, "sync")) return;
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
            busy={held(busy, "sync")}
            offered={state.cloudOffered && state.mode !== "folder"}
            onTrouble={(text) => (text ? fail({ card: "sync", text }) : cards.clear("sync"))}
            onDone={picked}
          />
          {trouble.sync && (
            <p role="alert" className="mt-3 text-[11.5px] text-urgent">
              {trouble.sync.text}
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
      <div
        id={PANEL}
        role="tabpanel"
        aria-label={finding ? t("findSetting") : t(pageOf(tab).label)}
        className="scroller min-h-0 flex-1"
      >
        <RowsContext.Provider value={rows}>
          <HeedContext.Provider value={shownItems}>
            <div className="mx-auto w-full max-w-[560px] px-6 pb-12">
              {finding ? (
                <Found asked={finding} onLand={land} />
              ) : (
                <PageHead tab={tab} onHelp={tab === "summary" ? undefined : onHelp} />
              )}

              {!finding && tab === "summary" && (
                <Summary
                  items={overview.items}
                  whole={overview.whole}
                  steps={overview.steps}
                  tiles={overview.tiles}
                  onLand={land}
                  extras={offering.extras}
                  onDeclined={offering.decline}
                />
              )}

              {!finding && (
                <KeepingPage
                  shelf={shelf}
                  landedAt={landedAt}
                  onStanding={lookAbout}
                  tab={tab}
                  onTab={onTab}
                  state={state}
                  kept={kept}
                  amend={amend}
                  build={build}
                  reach={reach}
                  wake={wake}
                  setWake={setWake}
                  keys={keys}
                  setKeys={setKeys}
                  audit={audit}
                  setAudit={setAudit}
                  freeing={freeing}
                  setFreeing={setFreeing}
                  freeDays={freeDays}
                  setFreeDays={setFreeDays}
                  seen={seen}
                  setSeen={setSeen}
                  holds={holds}
                  signs={signs}
                  known={known}
                  busy={busy}
                  run={run}
                  quietly={quietly}
                  tell={tell}
                  fail={fail}
                  clear={cards.clear}
                  carry={(way) => void carryNow(way)}
                  remember={remember}
                  pickFolder={pickFolder}
                  onMove={(resume) => setMoving({ resume })}
                  onPack={onPack}
                  onUnpack={onUnpack}
                  onChanged={onChanged}
                  onDoc={onDoc}
                  hidden={hidden}
                />
              )}
            </div>
          </HeedContext.Provider>
        </RowsContext.Provider>
      </div>
    </main>
  );
}

function Unread({ tab, trouble, retry }: { tab: Tab; trouble?: string; retry: () => void }) {
  return (
    <main className="flex flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div
        id={PANEL}
        role="tabpanel"
        aria-label={t(pageOf(tab).label)}
        className="mx-auto w-full max-w-[560px] px-6"
      >
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

function useAbout(greeted: number | undefined, look: () => void, answer: (read: number) => void) {
  const [reach, setReach] = useState<Reach | null>(null);
  const [wake, setWake] = useState<Waking | null>(null);
  const [keys, setKeys] = useState<string | null>(null);
  const [keysKnown, setKeysKnown] = useState(false);
  const [build, setBuild] = useState<About | null>(null);
  useEffect(() => {
    reachable()
      .then(setReach)
      .catch(() => {});
    waking()
      .then(setWake)
      .catch(() => {});
    shortcut()
      .then((now) => {
        setKeys(now);
        setKeysKnown(true);
      })
      .catch(() => {})
      .finally(() => answer(KEYS));
    about()
      .then(setBuild)
      .catch(() => {})
      .finally(() => answer(BUILD));
    look();
  }, [greeted, look, answer]);
  const lookAbout = useCallback(() => {
    about()
      .then(setBuild)
      .catch(() => {});
  }, []);
  const keysSet = useCallback((now: string | null) => {
    setKeys(now);
    setKeysKnown(true);
  }, []);
  return { reach, wake, setWake, keys, keysKnown, setKeys: keysSet, build, lookAbout };
}
