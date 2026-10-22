import type { ReactNode } from "react";
import { fill, t } from "../locales";

export type Which =
  | "sync"
  | "backup"
  | "restore"
  | "review"
  | "machines"
  | "quick"
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
  | "signing"
  | "naming"
  | "parcel"
  | "tongue"
  | "look"
  | "closing"
  | "beaver"
  | "rebuild"
  | "recover"
  | "missing"
  | "unread"
  | "twins"
  | "keys"
  | "loose"
  | "index";
/** `busy` marks a round turned away by another, whatever its words say. */
export type Word = { card: Which; text: string; busy?: boolean };

export type Run = <T>(card: Which, work: Promise<T>, then: (answer: T) => void) => void;

export const NAMED: Record<Which, Parameters<typeof t>[0]> = {
  sync: "syncing",
  signing: "alias",
  naming: "signName",
  parcel: "parcelsTitle",
  backup: "backup",
  restore: "restoreTitle",
  review: "review",
  machines: "theMachines",
  quick: "quick",
  waking: "wake",
  greet: "greetAgain",
  tongue: "tongue",
  closing: "closingSetting",
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
  unread: "unreadTitle",
  twins: "twinsAre",
  keys: "keysTitle",
  loose: "looseAre",
  index: "quickIndex",
};

const TAKING_LONG = 60;

export const working = (key: Parameters<typeof t>[0], secs: number): string =>
  secs >= TAKING_LONG ? fill("workingFor", t(key), String(Math.floor(secs / 60))) : t(key);

interface CardProps {
  title: string;
  which: Which;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  children: ReactNode;
}

export default function Card({ title, which, busy, said, trouble, children }: CardProps) {
  const waiting = busy !== null && busy !== which;
  return (
    <section className="mb-3 rounded-[10px] border border-hair px-4 py-3.5">
      <h3 className="mb-0.5 text-[13px] font-semibold">{title}</h3>
      {children}
      {waiting && (
        <p className="mt-2 text-[11.5px] text-faint">{fill("waitFor", t(NAMED[busy]))}</p>
      )}
      {trouble?.card === which && <p className="mt-2 text-[11.5px] text-urgent">{trouble.text}</p>}
      {said?.card === which && <p className="mt-2 text-[11.5px] text-faint">{said.text}</p>}
    </section>
  );
}
