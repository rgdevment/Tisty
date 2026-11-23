import type { ReactNode } from "react";
import type { About, Carrying, Freeing, Reach, Reviewed, Settings, Toward, Waking } from "../core";
import type { KnownAgents } from "../knownAgents";
import type { Shelf } from "../settingsRows";
import type { Tab } from "../views";
import Agents from "./Agents";
import type { Run, Which, Word } from "./Card";
import Notices from "./Notices";
import Security from "./Security";
import type { Signs } from "./Signing";
import Storage from "./Storage";
import Sync from "./Sync";
import Upkeep, { type Seen } from "./Upkeep";
import Window from "./Window";
import Writing from "./Writing";
import type { Working } from "./working";
import Yours from "./Yours";

export interface Shared {
  tab: Tab;
  onTab: (tab: Tab) => void;
  state: Carrying;
  kept: Settings | null;
  amend: (edit: (was: Settings) => Settings) => void;
  build: About | null;
  reach: Reach | null;
  wake: Waking | null;
  setWake: (wake: Waking) => void;
  keys: string | null;
  setKeys: (keys: string | null) => void;
  audit: Reviewed | null;
  setAudit: (audit: Reviewed) => void;
  freeing: Freeing | null;
  setFreeing: (freeing: Freeing | null) => void;
  freeDays: number;
  setFreeDays: (days: number) => void;
  seen: Seen | null;
  setSeen: (seen: Seen) => void;
  holds: string;
  signs: Signs;
  known: KnownAgents;
  busy: Working;
  run: Run;
  quietly: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  clear: (card: Which) => void;
  carry: (way?: "again") => void;
  remember: (next: Settings, card: Which) => void;
  pickFolder: () => void;
  onMove: (resume?: Toward) => void;
  onPack: () => void;
  onUnpack: () => void;
  onChanged: () => void;
  onDoc: (paper: string) => void;
  hidden?: ReactNode;
  shelf?: Shelf;
  landedAt?: number;
  onStanding?: () => void;
}

export default function KeepingPage(all: Shared) {
  const { busy, run, tell, fail, kept, state } = all;
  const cards = { busy, run, tell };
  switch (all.tab) {
    case "window":
      return (
        <Window
          {...cards}
          kept={kept}
          amend={all.amend}
          wake={all.wake}
          setWake={all.setWake}
          fail={fail}
          remember={all.remember}
          onChanged={all.onChanged}
          hidden={all.hidden}
        />
      );
    case "notices":
      return (
        <Notices
          {...cards}
          kept={kept}
          amend={all.amend}
          keys={all.keys}
          setKeys={all.setKeys}
          remember={all.remember}
        />
      );
    case "writing":
      return <Writing {...cards} kept={kept} amend={all.amend} signs={all.signs} />;
    case "sync":
      return (
        <Sync
          {...cards}
          state={state}
          kept={kept}
          fail={fail}
          carry={all.carry}
          pickFolder={all.pickFolder}
          onMove={all.onMove}
          onSecurity={() => all.onTab("security")}
        />
      );
    case "security":
      return (
        <Security
          {...cards}
          state={state}
          audit={all.audit}
          setAudit={all.setAudit}
          fail={fail}
          onChanged={all.onChanged}
          onIdentity={all.onStanding}
          onSync={() => all.onTab("sync")}
          carry={() => all.carry()}
          clear={all.clear}
        />
      );
    case "copies":
      return (
        <Yours
          {...cards}
          state={state}
          holds={all.holds}
          build={all.build}
          fail={fail}
          onPack={all.onPack}
          onUnpack={all.onUnpack}
        />
      );
    case "storage":
      return (
        <Storage
          {...cards}
          state={state}
          kept={kept}
          build={all.build}
          audit={all.audit}
          setAudit={all.setAudit}
          freeing={all.freeing}
          setFreeing={all.setFreeing}
          freeDays={all.freeDays}
          setFreeDays={all.setFreeDays}
          fail={fail}
          carry={() => all.carry()}
          remember={all.remember}
          clear={all.clear}
          onDoc={all.onDoc}
          onSync={() => all.onTab("sync")}
        />
      );
    case "agents":
      return (
        <Agents
          sent={all.shelf}
          sentAt={all.landedAt}
          busy={busy}
          quietly={all.quietly}
          tell={tell}
          fail={fail}
          binary={all.reach?.binary}
          known={all.known}
        />
      );
    case "health":
      return (
        <Upkeep
          {...cards}
          audit={all.audit}
          setAudit={all.setAudit}
          quietly={all.quietly}
          fail={fail}
          onSecurity={() => all.onTab("security")}
          seen={all.seen}
          setSeen={all.setSeen}
        />
      );
    default:
      return null;
  }
}
