import type { ReactNode } from "react";
import { fill, t } from "../locales";

export type Which =
  | "sync"
  | "backup"
  | "restore"
  | "review"
  | "machines"
  | "terminal"
  | "quick"
  | "waking"
  | "settings"
  | "notices"
  | "attach"
  | "holds"
  | "tagging"
  | "repeated"
  | "unvouched"
  | "wiring"
  | "report"
  | "store"
  | "brittle"
  | "greet"
  | "signing"
  | "parcel"
  | "tongue"
  | "look"
  | "closing";
export type Word = { card: Which; text: string };

export type Run = <T>(card: Which, work: Promise<T>, then: (answer: T) => void) => void;

export const NAMED: Record<Which, Parameters<typeof t>[0]> = {
  sync: "syncing",
  signing: "alias",
  parcel: "bandParcels",
  backup: "backup",
  restore: "restoreTitle",
  review: "review",
  machines: "theMachines",
  brittle: "brittleAre",
  terminal: "terminal",
  quick: "quick",
  waking: "wake",
  greet: "greetAgain",
  tongue: "tongue",
  closing: "closingSetting",
  look: "look",
  settings: "settingsTitle",
  notices: "bandNotices",
  attach: "attachTitle",
  holds: "holdsTitle",
  tagging: "tagsRead",
  repeated: "repeatedLists",
  unvouched: "unvouchedTitle",
  wiring: "wiringTitle",
  report: "reportTitle",
  store: "aboutStore",
};

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
