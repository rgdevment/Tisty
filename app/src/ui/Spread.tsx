import { useLayoutEffect, useMemo, useRef, useState } from "react";
import type { Task, Volume } from "../core";
import { clockOf } from "../format";
import { fill, locale, t } from "../locales";
import { HEAVY, weekday } from "./Ahead";

interface Props {
  tasks: Task[];
  onPlace: (task: string, on: string) => void;
  onOpen: (task: Task) => void;
}

const SLIP = 5;
const DOTS = 3;
const WEEK = 7;

const stamp = (at: Date): string =>
  `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, "0")}-${String(at.getDate()).padStart(2, "0")}`;

const monday = (now: Date): Date =>
  new Date(now.getFullYear(), now.getMonth(), now.getDate() - ((now.getDay() + 6) % 7));

const week = (now: Date): Date[] => {
  const first = monday(now);
  const out: Date[] = [];
  for (let i = 0; i < WEEK; i += 1) {
    out.push(new Date(first.getFullYear(), first.getMonth(), first.getDate() + i));
  }
  return out;
};

export const heftOf = (volume?: Volume): number => {
  const steps = volume?.steps ?? 0;
  const refs = volume?.refs ?? 0;
  const plan = steps <= 2 ? 0 : steps <= 7 ? 1 : 2;
  const linked = refs === 0 ? 0 : refs <= 2 ? 1 : 2;
  return (volume?.prose ?? 0) + plan + linked;
};

const sized = (heft: number): { word: string; lit: number } =>
  heft >= 4
    ? { word: t("spreadLarge"), lit: 3 }
    : heft >= 2
      ? { word: t("spreadMiddling"), lit: 2 }
      : { word: t("spreadSmall"), lit: 1 };

const named = (at: Date): string => weekday().format(at);

export default function Spread({ tasks, onPlace, onOpen }: Props) {
  const [over, setOver] = useState<string | null>(null);
  const [held, setHeld] = useState<string | null>(null);
  const [asked, setAsked] = useState<{ task: string; on: string; free?: string } | null>(null);
  const start = useRef<{ x: number; y: number } | null>(null);
  const ghost = useRef<HTMLDivElement>(null);
  const at = useRef({ x: 0, y: 0 });

  const now = useMemo(() => new Date(), []);
  const today = stamp(now);

  const spread = useMemo(() => {
    const loose = tasks.filter((one) => !one.repeat);
    return week(now).map((day) => {
      const key = stamp(day);
      const mine = loose.filter((one) => one.date?.at.slice(0, 10) === key);
      return {
        day,
        key,
        anchors: mine.filter((one) => one.date?.has_time),
        slots: mine.filter((one) => !one.date?.has_time),
        many: mine.length,
      };
    });
  }, [tasks, now]);

  const waiting = useMemo(
    () => tasks.filter((one) => !one.date && !one.deadline && !one.repeat),
    [tasks],
  );

  const trail = () => {
    const one = ghost.current;
    if (one) one.style.transform = `translate3d(${at.current.x + 14}px, ${at.current.y + 10}px, 0)`;
  };

  useLayoutEffect(trail, [held]);

  const under = (x: number, y: number): string | null =>
    document.elementFromPoint(x, y)?.closest("[data-day]")?.getAttribute("data-day") ?? null;

  const landed = (task: string, on: string) => {
    onPlace(task, on);
    const crowded = spread.find((one) => one.key === on);
    if (!crowded || crowded.many + 1 < HEAVY) return setAsked(null);
    setAsked({ task, on, free: spread.find((one) => one.key > on && one.many === 0)?.key });
  };

  const carry = (task: Task) => ({
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
      start.current = null;
      setHeld(null);
      setOver(null);
      if (!dragged) return onOpen(task);
      const where = under(e.clientX, e.clientY);
      if (where && where !== task.date?.at.slice(0, 10)) landed(task.id, where);
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
  const dayOf = (key: string) => spread.find((one) => one.key === key)?.day;

  return (
    <section
      className={`flex min-h-0 flex-1 flex-col gap-3 px-5 pt-4 pb-4 ${held ? "cursor-grabbing" : ""}`}
    >
      <h2 className="text-[19px] font-semibold tracking-[-0.015em]">
        {fill(
          "spreadWeek",
          String(spread[0]?.day.getDate() ?? ""),
          String(spread[WEEK - 1]?.day.getDate() ?? ""),
        )}{" "}
        <span className="text-[12.5px] font-normal text-faint">
          {new Intl.DateTimeFormat(locale(), { month: "long", year: "numeric" }).format(now)}
        </span>
      </h2>

      <div
        className="grid min-h-0 flex-1 gap-3"
        style={{ gridTemplateColumns: "228px minmax(0,1fr)" }}
      >
        <div className="flex min-h-0 flex-col gap-1.5 border-r border-hair pr-3">
          <p className="flex items-baseline justify-between text-[10px] font-semibold tracking-[0.06em] text-faint uppercase">
            <span>{t("spreadTray")}</span>
            <span className="tabular-nums">{waiting.length}</span>
          </p>
          {waiting.length === 0 ? (
            <p className="text-[11.5px] leading-relaxed text-faint">{t("spreadEmpty")}</p>
          ) : (
            <ul className="scroller flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto pr-1">
              {waiting.map((task) => {
                const heft = sized(heftOf(task.volume));
                return (
                  <li key={task.id}>
                    <button
                      type="button"
                      {...carry(task)}
                      className={`flex w-full flex-col gap-1 rounded-lg border border-hair bg-panel px-2 py-1.5 hover:bg-hover ${grip(task)}`}
                    >
                      <span className="w-full truncate text-[12px] leading-snug">{task.title}</span>
                      <span className="flex items-center gap-1.5 text-[9.5px] text-faint">
                        <span className="flex gap-px" aria-hidden="true">
                          {[0, 1, 2].map((one) => (
                            <span
                              key={one}
                              className={`block size-[4px] rounded-[1px] ${
                                one < heft.lit ? "bg-accent" : "bg-faint/30"
                              }`}
                            />
                          ))}
                        </span>
                        {heft.word}
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </div>

        <div className="flex min-h-0 gap-1 overflow-x-auto">
          {spread.map((one) => {
            const heavy = one.many >= HEAVY;
            return (
              <fieldset
                key={one.key}
                data-day={one.key}
                aria-label={`${named(one.day)} ${one.day.getDate()}`}
                className={`flex min-h-0 min-w-[104px] flex-1 flex-col gap-1 overflow-hidden rounded-xl border px-1.5 pb-1.5 transition-colors ${
                  over === one.key
                    ? "border-accent border-dashed bg-accent-soft ring-2 ring-accent/40"
                    : heavy
                      ? "border-hue-amber/40 bg-hue-amber/10"
                      : "border-hair bg-panel"
                }`}
              >
                <header className="flex items-baseline justify-between border-b border-hair pt-1.5 pb-1.5">
                  <span className="text-[9px] font-semibold tracking-[0.06em] text-faint uppercase">
                    {named(one.day)}{" "}
                    <span
                      className={`text-[11.5px] tabular-nums ${
                        one.key === today ? "text-accent" : heavy ? "text-hue-amber" : "text-ink"
                      }`}
                    >
                      {one.day.getDate()}
                    </span>
                  </span>
                  <span className="flex gap-px" aria-hidden="true">
                    {[0, 1, 2].map((at) => (
                      <span
                        key={at}
                        className={`block size-[3px] rounded-full ${
                          at < Math.min(one.many, DOTS)
                            ? heavy
                              ? "bg-hue-amber"
                              : "bg-accent"
                            : "bg-faint/25"
                        }`}
                      />
                    ))}
                  </span>
                </header>

                <ul className="scroller flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto">
                  {one.anchors.map((task) => (
                    <li key={task.id}>
                      <button
                        type="button"
                        {...carry(task)}
                        className={`flex w-full gap-1.5 rounded-[5px] bg-mark-date px-1.5 py-0.5 text-[10.5px] leading-snug hover:brightness-95 ${grip(task)}`}
                      >
                        {task.date && (
                          <span className="shrink-0 tabular-nums text-faint">
                            {clockOf(task.date)}
                          </span>
                        )}
                        <span className="min-w-0 truncate text-ink">{task.title}</span>
                      </button>
                    </li>
                  ))}
                  {one.slots.map((task) => (
                    <li key={task.id}>
                      <button
                        type="button"
                        {...carry(task)}
                        className={`flex w-full border-l-2 py-0.5 pl-1.5 text-[10.5px] leading-snug text-soft hover:text-ink ${
                          task.deadline ? "border-hue-amber" : "border-line"
                        } ${grip(task)}`}
                      >
                        <span className="min-w-0 truncate">{task.title}</span>
                      </button>
                    </li>
                  ))}
                  {one.many === 0 && (
                    <li className="mt-auto border-t border-dashed border-hair pt-1 text-[10px] text-faint italic">
                      {t("spreadFree")}
                    </li>
                  )}
                </ul>
              </fieldset>
            );
          })}
        </div>
      </div>

      {asked && (
        <p className="flex flex-wrap items-baseline gap-x-2 gap-y-1 border-l-2 border-hue-amber py-1 pl-2.5 text-[12px] leading-snug text-soft">
          <span className="text-ink">
            {fill(
              "spreadCrowded",
              named(dayOf(asked.on) ?? now),
              String((spread.find((one) => one.key === asked.on)?.many ?? 0) + 1),
            )}
          </span>
          {asked.free && (
            <>
              <span>{fill("spreadRoom", named(dayOf(asked.free) ?? now))}</span>
              <button
                type="button"
                onClick={() => {
                  if (asked.free) onPlace(asked.task, asked.free);
                  setAsked(null);
                }}
                className="text-accent hover:underline"
              >
                {fill("spreadMoveIt", named(dayOf(asked.free) ?? now))}
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
          className="shadow-lift-tall pointer-events-none fixed top-0 left-0 z-50 max-w-[240px] truncate rounded-md border border-accent bg-bg px-2 py-1 text-[11.5px]"
        >
          {carrying.title}
        </div>
      )}
    </section>
  );
}
