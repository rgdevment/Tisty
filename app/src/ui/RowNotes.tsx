import { createContext, type ReactNode, useContext, useEffect, useState } from "react";
import type { Item } from "../attention";
import { fill, t } from "../locales";
import type { RowId } from "../settingsRows";
import { type Heard, NAMED, type Which } from "./Card";
import { mild } from "./Rows";
import { holder, IDLE, type Working } from "./working";

export interface Rows {
  busy: Working;
  said: Heard;
  trouble: Heard;
}

export const RowsContext = createContext<Rows>({ busy: IDLE, said: {}, trouble: {} });
export const HeedContext = createContext<Item[] | null>(null);

export const useRows = (): Rows => useContext(RowsContext);

export type Kind = "info" | "warn" | "err" | "dep" | "sys" | "na";

const KIND: Record<Kind, Parameters<typeof t>[0]> = {
  info: "noteInfo",
  warn: "noteWarn",
  err: "noteErr",
  dep: "noteDep",
  sys: "noteSys",
  na: "noteNa",
};

const DRESS: Record<Kind, string> = {
  info: "bg-accent-soft text-soft",
  warn: "bg-hue-amber/10 text-soft",
  err: "bg-urgent/10 text-ink",
  dep: "border border-dashed border-line text-soft",
  sys: "border border-dashed border-line text-soft",
  na: "border border-dashed border-line text-soft",
};

const LABEL: Record<Kind, string> = {
  info: "text-accent",
  warn: "text-hue-amber",
  err: "text-urgent",
  dep: "text-faint",
  sys: "text-faint",
  na: "text-faint",
};

export function Note({
  kind,
  title,
  children,
  act,
}: {
  kind: Kind;
  title?: ReactNode;
  children?: ReactNode;
  act?: ReactNode;
}) {
  return (
    <div
      data-note={kind}
      className={`mt-2 flex flex-wrap items-start gap-x-2.5 gap-y-1.5 rounded-[10px] px-3 py-2 text-[12.5px] leading-snug ${DRESS[kind]}`}
    >
      <span
        className={`mt-0.5 w-[74px] shrink-0 text-[10.5px] font-semibold tracking-[0.04em] uppercase ${LABEL[kind]}`}
      >
        {t(KIND[kind])}
      </span>
      <span className="min-w-[12rem] flex-1">
        {title && <span className="block font-semibold text-ink">{title}</span>}
        {children}
      </span>
      {act && <span className="ml-auto flex shrink-0 flex-wrap gap-2">{act}</span>}
    </div>
  );
}

export function NoteButton({
  label,
  onPress,
  disabled,
}: {
  label: string;
  onPress: () => void;
  disabled?: boolean;
}) {
  return (
    <button type="button" disabled={disabled} onClick={onPress} className={`bg-bg ${mild}`}>
      {label}
    </button>
  );
}

const SHOWN_AFTER_MS = 400;

const DOING: Partial<Record<Which, Parameters<typeof t>[0]>> = {
  sync: "rowSyncing",
  backup: "rowBackingUp",
  restore: "rowRestoring",
  signing: "rowSigning",
  review: "rowChecking",
  rebuild: "rowRebuilding",
  index: "rowRebuilding",
  wiring: "rowWriting",
  report: "rowPreparing",
  machines: "rowWorking",
  sealing: "rowWorking",
};

export const doing = (which: Which): string => t(DOING[which] ?? "rowSaving");

function useSeconds(since: number | undefined): number | null {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (since === undefined) return;
    setNow(Date.now());
    const ticking = setInterval(() => setNow(Date.now()), 250);
    return () => clearInterval(ticking);
  }, [since]);
  if (since === undefined) return null;
  const spent = now - since;
  return spent < SHOWN_AFTER_MS ? null : Math.floor(spent / 1000);
}

export function Feedback({
  which,
  quiet,
  hideWorking,
  hideTrouble,
  hideSaid,
}: {
  which: Which;
  quiet?: boolean;
  hideWorking?: boolean;
  hideTrouble?: boolean;
  hideSaid?: boolean;
}) {
  const { busy, said, trouble } = useRows();
  const waitedOn = holder(busy, which);
  const secs = useSeconds(waitedOn === which && !hideWorking ? busy.get(which) : undefined);
  const waits = useSeconds(waitedOn && waitedOn !== which ? busy.get(waitedOn) : undefined);
  const told = hideSaid ? undefined : said[which];
  const wrong = hideTrouble ? undefined : trouble[which];
  return (
    <div data-feedback={which} aria-live="polite" className="text-[11.5px]">
      {secs !== null && (
        <p className="mt-1.5 text-faint" data-working="">
          {fill("rowWorkingFor", doing(which), String(secs))}
        </p>
      )}
      {waitedOn && waits !== null && !quiet && (
        <p className="mt-1.5 text-faint" data-waiting="">
          {fill("waitFor", t(NAMED[waitedOn]))}
        </p>
      )}
      {wrong && (
        <Note
          kind="err"
          act={wrong.again && <NoteButton label={t("tryAgain")} onPress={wrong.again} />}
        >
          {wrong.text}
        </Note>
      )}
      {told && !wrong && (
        <p className="mt-1.5 text-hue-green" data-said="">
          {told.text}
        </p>
      )}
    </div>
  );
}

export function Heeded({ row }: { row: RowId }) {
  const items = useContext(HeedContext);
  const here = items?.filter((one) => one.spot.row === row) ?? [];
  return (
    <>
      {here.map((one) => (
        <Note key={one.key} kind="warn" title={one.why && one.said}>
          {one.why ?? one.said}
        </Note>
      ))}
    </>
  );
}
