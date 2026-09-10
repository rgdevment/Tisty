import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { Task } from "../core";
import { clockOf } from "../format";
import { fill, locale, t } from "../locales";
import { HEAVY, weekday } from "./Ahead";
import Glyph from "./Glyph";

interface Props {
  tasks: Task[];
  onPlace: (task: string, on: string | null) => void;
  onOpen: (task: Task) => void;
  onCarrying?: (held: boolean) => void;
}

const SLIP = 5;
const WEEK = 7;
const RIVER = 120;
const DAY = 24 * 60 * 60 * 1000;
const GLANCE = 3;
const CELLS = 42;
const TRAY = "tray";

const stamp = (at: Date): string =>
  `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, "0")}-${String(at.getDate()).padStart(2, "0")}`;

const dayOne = (key: string): Date => {
  const [year, month, day] = key.split("-").map(Number);
  return new Date(year, month - 1, day);
};

const monday = (now: Date): Date =>
  new Date(now.getFullYear(), now.getMonth(), now.getDate() - ((now.getDay() + 6) % 7));

const walk = (from: Date, many: number): Date[] => {
  const out: Date[] = [];
  for (let i = 0; i < many; i += 1) {
    out.push(new Date(from.getFullYear(), from.getMonth(), from.getDate() + i));
  }
  return out;
};

const resting = (at: Date): boolean => at.getDay() === 0 || at.getDay() === 6;

const named = (at: Date): string => weekday().format(at);

const moonOf = (at: Date): string => {
  const said = new Intl.DateTimeFormat(locale(), { month: "long", year: "numeric" }).format(at);
  return said.charAt(0).toUpperCase() + said.slice(1);
};

export default function Spread({ tasks, onPlace, onOpen, onCarrying }: Props) {
  const [glance, setGlance] = useState<{ year: number; month: number } | null>(null);
  const [over, setOver] = useState<string | null>(null);
  const [held, setHeld] = useState<string | null>(null);
  const [asked, setAsked] = useState<{
    task: string;
    on: string;
    many: number;
    free?: string;
  } | null>(null);
  const start = useRef<{ x: number; y: number } | null>(null);
  const slid = useRef(false);
  const ghost = useRef<HTMLDivElement>(null);
  const river = useRef<HTMLDivElement>(null);
  const wanted = useRef<string | null>(null);
  const at = useRef({ x: 0, y: 0 });

  const now = new Date();
  const today = stamp(now);

  const carried = useMemo(() => {
    const held = new Map<string, Task[]>();
    const put = (key: string, task: Task) => {
      const mine = held.get(key);
      if (mine) mine.push(task);
      else held.set(key, [task]);
    };
    for (const task of tasks) {
      if (task.repeat) continue;
      const on = task.date?.at.slice(0, 10);
      const owed = task.deadline?.at.slice(0, 10);
      if (on) put(on, task);
      if (owed && owed !== on) put(owed, task);
    }
    for (const mine of held.values()) {
      mine.sort((a, b) => (a.date?.at ?? "").localeCompare(b.date?.at ?? ""));
    }
    return held;
  }, [tasks]);

  const days = useMemo(() => {
    const here = monday(dayOne(today));
    const oldest = [...carried.keys()].sort()[0];
    const from = oldest && oldest < stamp(here) ? monday(dayOne(oldest)) : here;
    const span = Math.round((here.getTime() - from.getTime()) / DAY) + RIVER;
    return walk(from, span);
  }, [today, carried]);

  const waiting = useMemo(
    () => tasks.filter((one) => !one.date && !one.deadline && !one.repeat),
    [tasks],
  );

  const reach = useCallback((key: string) => {
    river.current?.querySelector(`[data-day="${key}"]`)?.scrollIntoView?.({ block: "start" });
  }, []);

  useEffect(() => {
    if (glance !== null) return;
    const key = wanted.current;
    wanted.current = null;
    if (key) reach(key);
    else if (river.current) river.current.scrollTop = 0;
  }, [glance, reach]);

  const trail = () => {
    const one = ghost.current;
    if (one) one.style.transform = `translate3d(${at.current.x + 14}px, ${at.current.y + 10}px, 0)`;
  };

  useLayoutEffect(trail, [held]);

  useEffect(() => {
    onCarrying?.(held !== null);
  }, [held, onCarrying]);

  const under = (x: number, y: number): string | null => {
    const spot = document.elementFromPoint(x, y);
    if (spot?.closest("[data-tray]")) return TRAY;
    if (glance) return null;
    return spot?.closest("[data-day]")?.getAttribute("data-day") ?? null;
  };

  const kept = (task: Task | undefined, on: string): string =>
    task?.date?.has_time ? `${on}${task.date.at.slice(10)}` : on;

  const landed = (task: string, on: string) => {
    onPlace(
      task,
      kept(
        tasks.find((one) => one.id === task),
        on,
      ),
    );
    const many = (carried.get(on)?.length ?? 0) + 1;
    if (many < HEAVY) return setAsked(null);
    const free = days.find(
      (day) => stamp(day) > on && stamp(day) >= today && !carried.get(stamp(day))?.length,
    );
    setAsked({ task, on, many, free: free && stamp(free) });
  };

  const carry = (task: Task) => ({
    onClick: () => {
      if (slid.current) {
        slid.current = false;
        return;
      }
      onOpen(task);
    },
    onPointerDown: (e: React.PointerEvent) => {
      if (e.button !== 0) return;
      start.current = { x: e.clientX, y: e.clientY };
      e.currentTarget.setPointerCapture(e.pointerId);
    },
    onPointerMove: (e: React.PointerEvent) => {
      const began = start.current;
      if (!began) return;
      if (!held && Math.hypot(e.clientX - began.x, e.clientY - began.y) < SLIP) return;
      at.current = { x: e.clientX, y: e.clientY };
      trail();
      setHeld(task.id);
      setOver(under(e.clientX, e.clientY));
    },
    onPointerUp: (e: React.PointerEvent) => {
      const dragged = held === task.id;
      slid.current = dragged;
      start.current = null;
      setHeld(null);
      setOver(null);
      if (!dragged) return;
      const where = under(e.clientX, e.clientY);
      if (!where) return;
      if (where === TRAY) {
        if (task.date) {
          onPlace(task.id, null);
          setAsked(null);
        }
        return;
      }
      if (where !== task.date?.at.slice(0, 10)) landed(task.id, where);
    },
    onPointerCancel: () => {
      start.current = null;
      setHeld(null);
      setOver(null);
    },
  });

  const grip = (task: Task) =>
    `touch-none text-left select-none ${held === task.id ? "cursor-grabbing opacity-30" : "cursor-grab"}`;

  const carrying = held ? tasks.find((one) => one.id === held) : undefined;
  const losing = carrying?.date !== undefined;

  const card = (task: Task) => (
    <button
      key={task.id}
      type="button"
      {...carry(task)}
      className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-[13px] hover:bg-hover ${grip(task)}`}
    >
      <span
        aria-hidden="true"
        className={`size-3.5 shrink-0 rounded-full border-[1.5px] ${
          task.deadline ? "border-hue-amber" : "border-line"
        }`}
      />
      {task.date?.has_time && (
        <span className="shrink-0 text-[11.5px] tabular-nums text-faint">{clockOf(task.date)}</span>
      )}
      <span className="min-w-0 truncate">{task.title}</span>
    </button>
  );

  const flowing = () => {
    const out: React.ReactNode[] = [];
    let moon = -1;
    for (const day of days) {
      const key = stamp(day);
      const mine = carried.get(key) ?? [];
      if (day.getMonth() !== moon) {
        moon = day.getMonth();
        out.push(
          <p
            key={`moon-${key}`}
            className="sticky top-0 z-10 shrink-0 bg-desk px-0.5 pt-2 pb-1 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase"
          >
            {moonOf(day)}
          </p>,
        );
      }
      out.push(
        <div
          key={key}
          data-day={key}
          className={`grid shrink-0 scroll-mt-8 grid-cols-[84px_minmax(0,1fr)] overflow-hidden rounded-[10px] border transition-colors ${
            over === key
              ? "border-accent bg-accent-soft ring-2 ring-accent/40"
              : key === today
                ? "border-accent/40 bg-panel"
                : resting(day)
                  ? "border-hair border-dashed"
                  : "border-hair bg-panel"
          }`}
        >
          <p
            className={`flex min-h-9 flex-col justify-center border-r border-hair px-3 py-1.5 ${
              key === today ? "text-accent" : ""
            }`}
          >
            <span className="text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase">
              {named(day)}
            </span>
            <span className="text-[13px] font-semibold tabular-nums">{day.getDate()}</span>
          </p>
          {mine.length === 0 ? (
            <p className="flex min-h-9 items-center px-3 text-[12.5px] text-faint italic">
              {t("spreadFree")}
            </p>
          ) : (
            <div className="flex min-h-9 flex-col justify-center gap-px p-1.5">
              {mine.map(card)}
            </div>
          )}
        </div>,
      );
    }
    return out;
  };

  const glanced = () => {
    if (!glance) return null;
    const first = new Date(glance.year, glance.month, 1);
    const from = new Date(glance.year, glance.month, 1 - ((first.getDay() + 6) % 7));
    return walk(from, CELLS).map((day) => {
      const key = stamp(day);
      const mine = carried.get(key) ?? [];
      const away = day.getMonth() !== glance.month;
      return (
        <button
          key={key}
          type="button"
          data-day={key}
          aria-label={[moonOf(day), String(day.getDate()), ...mine.map((one) => one.title)].join(
            " ",
          )}
          onClick={() => {
            wanted.current = key;
            setGlance(null);
          }}
          className={`flex min-w-0 flex-col gap-0.5 overflow-hidden rounded-[10px] border px-1.5 py-1 text-left hover:border-line ${
            key === today
              ? "border-accent/40 bg-panel"
              : away || resting(day)
                ? "border-hair border-dashed"
                : "border-hair bg-panel"
          }`}
        >
          <span
            className={`text-[11.5px] tabular-nums ${
              key === today ? "font-semibold text-accent" : away ? "text-faint/40" : "text-soft"
            }`}
          >
            {day.getDate()}
          </span>
          {mine.slice(0, GLANCE).map((task) => (
            <span key={task.id} className="flex min-w-0 items-center gap-1 text-[10.5px] text-soft">
              <span
                aria-hidden="true"
                className={`size-1 shrink-0 rounded-full border ${
                  task.deadline ? "border-hue-amber" : "border-line"
                }`}
              />
              <span className="min-w-0 truncate">{task.title}</span>
            </span>
          ))}
          {mine.length > GLANCE && (
            <span className="mt-auto text-[10.5px] text-faint">+{mine.length - GLANCE}</span>
          )}
        </button>
      );
    });
  };

  const swung = (by: number) =>
    setGlance((was) => {
      if (!was) return was;
      const at = new Date(was.year, was.month + by, 1);
      return { year: at.getFullYear(), month: at.getMonth() };
    });

  return (
    <section
      className={`flex min-h-0 flex-1 flex-col gap-3 px-5 pb-4 ${held ? "cursor-grabbing" : ""}`}
    >
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <header className="-mt-3 flex items-center gap-1">
        <h2 className="mr-1 text-[21px] font-semibold tracking-[-0.015em]">
          {glance ? moonOf(new Date(glance.year, glance.month, 1)) : t("spread")}
        </h2>
        {glance && (
          <>
            <button
              type="button"
              aria-label={t("spreadBack")}
              onClick={() => swung(-1)}
              className="grid size-6 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
            >
              <Glyph name="chevron" className="h-3.5 w-3.5 rotate-90" />
            </button>
            <button
              type="button"
              aria-label={t("spreadOn")}
              onClick={() => swung(1)}
              className="grid size-6 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
            >
              <Glyph name="chevron" className="h-3.5 w-3.5 -rotate-90" />
            </button>
          </>
        )}
        <div className="ml-auto flex items-center gap-1">
          <button
            type="button"
            onClick={() => {
              if (!glance) return reach(today);
              wanted.current = today;
              setGlance(null);
            }}
            className="rounded-md px-2 py-1 text-[11.5px] text-accent hover:bg-hover"
          >
            {t("spreadNow")}
          </button>
          <button
            type="button"
            aria-pressed={glance !== null}
            onClick={() =>
              setGlance((was) => (was ? null : { year: now.getFullYear(), month: now.getMonth() }))
            }
            className={`rounded-md px-2 py-1 text-[11.5px] hover:bg-hover ${
              glance ? "bg-hover text-ink" : "text-faint hover:text-ink"
            }`}
          >
            {t("spreadMoon")}
          </button>
        </div>
      </header>

      <div
        className="grid min-h-0 flex-1 gap-3"
        style={{ gridTemplateColumns: glance ? "minmax(0,1fr)" : "232px minmax(0,1fr)" }}
      >
        {!glance && (
          <section
            data-tray=""
            className={`flex min-h-0 flex-col overflow-hidden rounded-[10px] border border-dashed transition-colors ${
              over === TRAY && losing ? "border-accent bg-accent-soft" : "border-line bg-panel"
            }`}
          >
            <header className="flex items-center gap-2 border-b border-hair px-3 py-2">
              <span className="text-[13px] font-semibold">{t("spreadTray")}</span>
              <span className="ml-auto text-[11.5px] tabular-nums text-faint">
                {waiting.length || ""}
              </span>
            </header>
            {losing ? (
              <p className="grid flex-1 place-items-center px-3 text-center text-[12.5px] text-accent">
                {t("spreadOff")}
              </p>
            ) : waiting.length === 0 ? (
              <p className="grid flex-1 place-items-center px-3 text-center text-[12.5px] leading-relaxed text-faint">
                {t("spreadEmpty")}
              </p>
            ) : (
              <div className="scroller flex-1 p-1.5">{waiting.map(card)}</div>
            )}
          </section>
        )}

        {glance ? (
          <div className="flex min-h-0 flex-col gap-1.5">
            <div className="grid grid-cols-7 gap-1.5">
              {walk(monday(now), WEEK).map((day) => (
                <span
                  key={day.getDay()}
                  className="pl-1 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase"
                >
                  {named(day)}
                </span>
              ))}
            </div>
            <div className="grid min-h-0 flex-1 auto-rows-fr grid-cols-7 gap-1.5">{glanced()}</div>
          </div>
        ) : (
          <div ref={river} className="scroller flex min-h-0 flex-col gap-1.5 pr-1">
            {flowing()}
          </div>
        )}
      </div>

      {asked && (
        <p className="flex flex-wrap items-baseline gap-x-2 gap-y-1 border-l-2 border-hue-amber py-1 pl-2.5 text-[12.5px] leading-snug text-soft">
          <span className="text-ink">
            {fill("spreadCrowded", named(dayOne(asked.on)), String(asked.many))}
          </span>
          {asked.free && (
            <>
              <span>{fill("spreadRoom", named(dayOne(asked.free)))}</span>
              <button
                type="button"
                onClick={() => {
                  if (asked.free) {
                    onPlace(
                      asked.task,
                      kept(
                        tasks.find((one) => one.id === asked.task),
                        asked.free,
                      ),
                    );
                  }
                  setAsked(null);
                }}
                className="text-accent hover:underline"
              >
                {fill("spreadMoveIt", named(dayOne(asked.free)))}
              </button>
            </>
          )}
          <button
            type="button"
            onClick={() => setAsked(null)}
            className="text-faint hover:text-ink"
          >
            {t("spreadLeaveIt")}
          </button>
        </p>
      )}

      {carrying && (
        <div
          ref={ghost}
          className="shadow-lift-tall pointer-events-none fixed top-0 left-0 z-50 max-w-[240px] truncate rounded-md border border-accent bg-bg px-2 py-1 text-[12.5px]"
        >
          {carrying.title}
        </div>
      )}
    </section>
  );
}
