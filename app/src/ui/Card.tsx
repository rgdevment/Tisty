import type { ReactNode } from "react";
import { fill, t } from "../locales";
import { ROWS, type RowId, settingAt } from "../settingsRows";
import { Feedback, Heeded } from "./RowNotes";

export type Which =
  | "sync"
  | "backup"
  | "restore"
  | "review"
  | "machines"
  | "quick"
  | "jot"
  | "waking"
  | "settings"
  | "notices"
  | "attach"
  | "holds"
  | "free"
  | "repeated"
  | "unvouched"
  | "wiring"
  | "report"
  | "store"
  | "greet"
  | "site"
  | "signing"
  | "naming"
  | "parcel"
  | "tongue"
  | "look"
  | "closing"
  | "calendar"
  | "beaver"
  | "rebuild"
  | "recover"
  | "missing"
  | "hidden"
  | "unread"
  | "twins"
  | "keys"
  | "loose"
  | "index"
  | "identity"
  | "sealing"
  | "earlier"
  | "syncOff"
  | "screen"
  | "chime"
  | "urgent"
  | "door"
  | "asks"
  | "hooks"
  | "cloudForget"
  | "cloudRevoke"
  | "cloudAbandon"
  | "sealWipe"
  | "sealOff"
  | "update"
  | "candidates";
/** `busy` marks a round turned away by another, whatever its words say. */
export type Word = { card: Which; text: string; busy?: boolean; again?: () => void };

export type Heard = Partial<Record<Which, Word>>;

export type Run = <T>(
  card: Which,
  work: Promise<T> | (() => Promise<T>),
  then: (answer: T) => void,
) => void;

export const NAMED: Record<Which, Parameters<typeof t>[0]> = {
  sync: "syncing",
  earlier: "earlierBand",
  signing: "alias",
  naming: "signName",
  parcel: "parcelsTitle",
  backup: "backup",
  restore: "restoreTitle",
  review: "review",
  machines: "theMachines",
  quick: "quick",
  jot: "quickIdea",
  waking: "wake",
  greet: "greetAgain",
  site: "helpOnSite",
  tongue: "tongue",
  closing: "closingSetting",
  calendar: "calendarSetting",
  beaver: "beaverPeeks",
  look: "look",
  settings: "settingsTitle",
  notices: "bandNotices",
  attach: "attachTitle",
  holds: "holdsTitle",
  free: "freeTitle",
  repeated: "repeatedLists",
  unvouched: "unvouchedTitle",
  wiring: "wiringTitle",
  report: "reportTitle",
  store: "aboutStore",
  rebuild: "quickIndex",
  recover: "upkeepLook",
  missing: "missingTitle",
  hidden: "hiddenTitle",
  unread: "unreadTitle",
  twins: "twinsAre",
  keys: "keysTitle",
  loose: "looseAre",
  index: "quickIndex",
  identity: "tabIdentity",
  sealing: "sealTitle",
  syncOff: "syncOffTitle",
  screen: "noticeScreen",
  chime: "noticeChime",
  urgent: "noticeUrgent",
  door: "agentsTitle",
  asks: "agentsAsk",
  hooks: "agentsHooks",
  cloudForget: "cloudForgetHere",
  cloudRevoke: "cloudRevokeAll",
  cloudAbandon: "cloudAbandon",
  sealWipe: "sealWipe",
  sealOff: "sealOff",
  update: "lookNow",
  candidates: "betaTake",
};

const TAKING_LONG = 60;

export const working = (key: Parameters<typeof t>[0], secs: number): string =>
  secs >= TAKING_LONG ? fill("workingFor", t(key), String(Math.floor(secs / 60))) : t(key);

interface CardProps {
  row?: RowId;
  item?: boolean;
  title?: string;
  which: Which;
  note?: ReactNode;
  children: ReactNode;
}

export default function Card({ row, item, title, which, note, children }: CardProps) {
  return (
    <section
      data-card=""
      data-item={item ? "" : undefined}
      {...(row && settingAt(row))}
      className="mb-3 rounded-[10px] border border-hair px-4 py-3.5"
    >
      <h3 className="mb-0.5 text-[13px] font-semibold">{title ?? (row && t(ROWS[row].title))}</h3>
      {children}
      {note}
      {row && <Heeded row={row} />}
      <Feedback which={which} />
    </section>
  );
}
