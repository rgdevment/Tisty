import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import {
  type IcsBrought,
  type IcsFound,
  type IcsOdd,
  icsRead,
  icsTake,
  icsWaiting,
  type List,
} from "../core";
import { useDesk } from "../desk";
import { cadence, whenLabel } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";

export const CALENDAR_DROPPED = "tisty:calendar";

// Past this many, choosing one by one needs a way to choose them all at once.
const MANY = 4;

const ODD: Record<IcsOdd, Parameters<typeof t>[0]> = {
  untitled: "icsUntitled",
  moved: "icsMoved",
  cancelled: "icsCancelled",
  repeat: "icsRepeatOdd",
  zone: "icsZoneOdd",
  past: "icsPast",
};

// A file opened while Tisty was starting waits for the window, so nothing is lost on the way in.
export default function Calendars() {
  const { data, load, setError } = useDesk();
  const [queue, setQueue] = useState<string[]>([]);

  useEffect(() => {
    const more = (paths: string[]) => {
      if (paths.length) setQueue((was) => [...was, ...paths]);
    };
    const waiting = () => icsWaiting().then(more, () => {});
    waiting();
    const heard = listen("calendar", waiting);
    const dropped = (event: Event) => more((event as CustomEvent<string[]>).detail ?? []);
    window.addEventListener(CALENDAR_DROPPED, dropped);
    return () => {
      heard.then((off) => off()).catch(() => {});
      window.removeEventListener(CALENDAR_DROPPED, dropped);
    };
  }, []);

  const path = queue[0];
  if (!path) return null;
  return (
    <Importing
      key={path}
      path={path}
      lists={data.lists.filter((one) => !one.archived)}
      onDone={(made) => {
        setQueue((was) => was.slice(1));
        if (made > 0) load();
      }}
      fail={(e) => setError(saidPlainly(e))}
    />
  );
}

export function Importing({
  path,
  lists,
  onDone,
  fail,
}: {
  path: string;
  lists: List[];
  onDone: (made: number) => void;
  fail: (e: unknown) => void;
}) {
  const [brought, setBrought] = useState<IcsBrought | null>(null);
  const [picked, setPicked] = useState<Set<number>>(new Set());
  const [list, setList] = useState("");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    icsRead(path).then(
      (said) => {
        setBrought(said);
        const ready = said.found.filter((one) => one.takeable && !one.odd.includes("past"));
        setPicked(new Set(ready.map((one) => one.at)));
      },
      (e) => setProblem(saidPlainly(e)),
    );
  }, [path]);

  const takeable = brought?.found.filter((one) => one.takeable) ?? [];
  const toggle = (at: number) =>
    setPicked((was) => {
      const next = new Set(was);
      if (next.has(at)) next.delete(at);
      else next.add(at);
      return next;
    });
  const take = () => {
    setBusy(true);
    icsTake(path, [...picked], list || null).then(
      (made) => onDone(made),
      (e) => {
        setBusy(false);
        fail(e);
      },
    );
  };

  return (
    <Modal title={t("icsTitle")} wide onClose={() => onDone(0)}>
      {problem && <p className="text-[13px] text-hue-red">{problem}</p>}
      {!brought && !problem && <p className="text-[13px] text-faint">{t("icsReading")}</p>}
      {brought && brought.found.length === 0 && (
        <p className="text-[13px] text-faint">{t("icsNothing")}</p>
      )}
      {brought?.invitation && <p className="mb-3 text-[12.5px] text-soft">{t("icsInvitation")}</p>}
      {takeable.length >= MANY && (
        <div className="mb-2 flex gap-3 text-[12.5px]">
          <button
            type="button"
            onClick={() => setPicked(new Set(takeable.map((one) => one.at)))}
            className="cursor-pointer text-accent hover:underline"
          >
            {t("icsAll")}
          </button>
          <button
            type="button"
            onClick={() => setPicked(new Set())}
            className="cursor-pointer text-accent hover:underline"
          >
            {t("icsNone")}
          </button>
        </div>
      )}
      {brought && brought.found.length > 0 && (
        <ul className="max-h-[50vh] overflow-y-auto rounded-[10px] border border-hair">
          {brought.found.map((one) => (
            <Row key={one.at} one={one} on={picked.has(one.at)} onToggle={() => toggle(one.at)} />
          ))}
        </ul>
      )}
      <div className="mt-4 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
        <label className="mr-auto flex items-center gap-2 text-soft">
          {t("icsIntoList")}
          <select
            value={list}
            onChange={(e) => setList(e.target.value)}
            className="min-w-0 max-w-[60%] truncate rounded-md border border-line bg-bg px-2 py-1 text-[12.5px] text-ink"
          >
            <option value="">{t("noList")}</option>
            {lists.map((one) => (
              <option key={one.id} value={one.id}>
                {one.name}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          onClick={() => onDone(0)}
          className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
        >
          {t("cancel")}
        </button>
        <button
          type="button"
          disabled={busy || picked.size === 0}
          onClick={take}
          className="cursor-pointer rounded-[10px] border border-line px-3 py-1.5 text-ink hover:bg-line/40 disabled:cursor-default disabled:opacity-50"
        >
          {fill("icsTake", String(picked.size))}
        </button>
      </div>
    </Modal>
  );
}

function Row({ one, on, onToggle }: { one: IcsFound; on: boolean; onToggle: () => void }) {
  const when = one.date ?? one.deadline;
  const notes = [
    one.already ? t("icsAlready") : null,
    one.done ? t("icsDone") : null,
    ...one.odd.map((odd) => t(ODD[odd])),
  ].filter(Boolean);
  return (
    <li className="border-hair border-t first:border-t-0">
      <label
        className={`flex items-start gap-2.5 px-3 py-2 text-[13px] ${one.takeable ? "cursor-pointer hover:bg-hover" : "opacity-60"}`}
      >
        <input
          type="checkbox"
          checked={on}
          disabled={!one.takeable}
          onChange={onToggle}
          className="mt-0.5 accent-accent"
        />
        <span className="min-w-0 flex-1">
          <span className="block truncate font-medium text-ink">
            {one.title || t("icsUntitled")}
          </span>
          <span className="block text-[12px] text-soft">
            {[
              when ? whenLabel(when) : null,
              one.place || null,
              one.repeat ? cadence(one.repeat) : null,
              one.reminder ? fill("icsReminder", whenLabel(one.reminder)) : null,
            ]
              .filter(Boolean)
              .join(" · ")}
          </span>
          {notes.length > 0 && (
            <span className="block text-[12px] text-hue-amber">{notes.join(" · ")}</span>
          )}
        </span>
      </label>
    </li>
  );
}
