import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { List, Priority, Task } from "../core";
import { fill, t } from "../locales";
import { QUADRANTS, said, tint } from "../quadrants";
import Only from "./Only";

const WIDE = 1280;
const SLIP = 5;

interface Props {
  tasks: Task[];
  lists: List[];
  beside: boolean;
  onPlace: (task: string, where: Priority) => void;
  onOpen: (task: Task) => void;
  onSow: (where: Priority) => void;
  onDiscardAll: (tasks: string[]) => void;
}

const KEPT = "tisty.tray";

export default function Matrix({
  tasks,
  lists,
  beside,
  onPlace,
  onOpen,
  onSow,
  onDiscardAll,
}: Props) {
  const [asked, setAsked] = useState(() => localStorage.getItem(KEPT) === "open");
  const [only, setOnly] = useState<string[]>([]);
  const [over, setOver] = useState<Priority | null>(null);
  const [held, setHeld] = useState<string | null>(null);
  const crossed = useRef(window.innerWidth >= WIDE);
  const from = useRef<{ x: number; y: number } | null>(null);
  const ghost = useRef<HTMLDivElement>(null);
  const at = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const look = () => {
      const wide = window.innerWidth >= WIDE;
      if (crossed.current === wide) return;
      crossed.current = wide;
      setAsked(wide && localStorage.getItem(KEPT) === "open");
    };
    window.addEventListener("resize", look);
    return () => window.removeEventListener("resize", look);
  }, []);

  const trail = () => {
    const one = ghost.current;
    if (one) one.style.transform = `translate3d(${at.current.x + 14}px, ${at.current.y + 10}px, 0)`;
  };

  useLayoutEffect(trail, [held]);

  const tray = asked && !beside;

  const swing = (open: boolean) => {
    localStorage.setItem(KEPT, open ? "open" : "shut");
    setAsked(open);
  };

  const placed = useMemo(() => {
    const by = new Map<Priority, Task[]>(QUADRANTS.map((one) => [one, [] as Task[]]));
    for (const task of tasks) {
      if (task.priority !== "unset") by.get(task.priority)?.push(task);
    }
    return by;
  }, [tasks]);

  const loose = useMemo(
    () => tasks.filter((task) => task.priority === "unset" && !task.repeat),
    [tasks],
  );

  const unplaced = useMemo(
    () => loose.filter((task) => only.length === 0 || (task.list && only.includes(task.list))),
    [loose, only],
  );

  const waiting = loose.length;

  const under = (x: number, y: number): Priority | null => {
    const zone = document.elementFromPoint(x, y)?.closest("[data-quadrant]");
    return (zone?.getAttribute("data-quadrant") as Priority | undefined) ?? null;
  };

  const carry = (task: Task) => ({
    onPointerDown: (e: React.PointerEvent) => {
      if (e.button !== 0) return;
      from.current = { x: e.clientX, y: e.clientY };
      e.currentTarget.setPointerCapture(e.pointerId);
    },
    onPointerMove: (e: React.PointerEvent) => {
      const start = from.current;
      if (!start) return;
      if (!held && Math.hypot(e.clientX - start.x, e.clientY - start.y) < SLIP) return;
      at.current = { x: e.clientX, y: e.clientY };
      trail();
      setHeld(task.id);
      setOver(under(e.clientX, e.clientY));
    },
    onPointerUp: (e: React.PointerEvent) => {
      const dragged = held === task.id;
      from.current = null;
      setHeld(null);
      setOver(null);
      if (!dragged) return onOpen(task);
      const where = under(e.clientX, e.clientY);
      if (where && where !== task.priority) onPlace(task.id, where);
    },
    onPointerCancel: () => {
      from.current = null;
      setHeld(null);
      setOver(null);
    },
  });

  const card = (task: Task) => (
    <li key={task.id}>
      <button
        type="button"
        {...carry(task)}
        className={`flex w-full touch-none items-center gap-2 rounded-md px-2 py-1.5 text-left text-[13px] select-none hover:bg-hover ${
          held === task.id ? "cursor-grabbing opacity-30" : "cursor-grab"
        }`}
      >
        <span
          aria-hidden
          className="h-3.5 w-3.5 shrink-0 rounded-full border-[1.5px] border-line"
        />
        <span className="truncate">{task.title}</span>
      </button>
    </li>
  );

  const carrying = held ? tasks.find((one) => one.id === held) : undefined;

  return (
    <section
      className={`flex min-h-0 flex-1 flex-col gap-3 px-5 pt-4 pb-5 ${held ? "cursor-grabbing" : ""}`}
    >
      <header className="flex items-baseline gap-3">
        <h2 className="text-[19px] font-semibold tracking-[-0.015em]">{t("quadrants")}</h2>
        {!asked && !beside && waiting > 0 && (
          <button
            type="button"
            onClick={() => swing(true)}
            className="rounded-full border border-line px-2.5 py-0.5 text-[11.5px] text-faint hover:text-soft"
          >
            {t("showUnplaced")} · {waiting}
          </button>
        )}
      </header>

      <div
        className="grid min-h-0 flex-1 gap-3"
        style={{ gridTemplateColumns: tray ? "minmax(0,1fr) 288px" : "minmax(0,1fr)" }}
      >
        <div className="grid min-h-0 grid-cols-[26px_minmax(0,1fr)] grid-rows-[26px_minmax(0,1fr)] gap-2">
          <div className="col-start-2 grid grid-cols-2 gap-3">
            <Axis label={t("urgentAxis")} />
            <Axis label={t("notUrgentAxis")} />
          </div>
          <div className="row-start-2 grid grid-rows-2 gap-3">
            <Axis label={t("importantAxis")} down />
            <Axis label={t("notImportantAxis")} down />
          </div>

          <div className="col-start-2 row-start-2 grid min-h-0 grid-cols-2 grid-rows-2 gap-3">
            {QUADRANTS.map((where) => {
              const mine = placed.get(where) ?? [];
              return (
                <fieldset
                  key={where}
                  data-quadrant={where}
                  aria-label={said(where)}
                  className={`flex min-h-0 min-w-0 flex-col overflow-hidden rounded-xl border transition-colors ${
                    over === where
                      ? "border-accent bg-accent-soft ring-2 ring-accent/40"
                      : held
                        ? "border-line border-dashed bg-panel"
                        : "border-hair bg-panel"
                  }`}
                >
                  <header className="flex items-center gap-2 border-b border-hair px-3 py-2">
                    <span className={`text-[13px] font-semibold ${tint(where)}`}>
                      {said(where)}
                    </span>
                    <span className="ml-auto text-[11.5px] text-faint tabular-nums">
                      {mine.length || ""}
                    </span>
                    {where === "minor" ? (
                      mine.length > 0 && (
                        <button
                          type="button"
                          onClick={() => onDiscardAll(mine.map((one) => one.id))}
                          className="rounded px-1.5 py-0.5 text-[11.5px] text-faint hover:bg-hover hover:text-soft"
                        >
                          {t("dropThemAll")}
                        </button>
                      )
                    ) : (
                      <Sow where={where} onSow={onSow} />
                    )}
                  </header>
                  {mine.length === 0 ? (
                    <p className="grid flex-1 place-items-center text-[12.5px] text-faint">
                      {t("placeItHere")}
                    </p>
                  ) : (
                    <ul className="scroller flex-1 p-1.5">{mine.map(card)}</ul>
                  )}
                </fieldset>
              );
            })}
          </div>
        </div>

        {tray && (
          <aside
            data-quadrant="unset"
            aria-label={t("unplaced")}
            className={`flex min-h-0 flex-col rounded-xl border border-dashed transition-colors ${
              over === "unset"
                ? "border-accent bg-accent-soft ring-2 ring-accent/40"
                : "border-line bg-panel"
            }`}
          >
            <header className="flex items-center gap-2 border-b border-hair px-3 py-2">
              <strong className="text-[13px] font-semibold">{t("unplaced")}</strong>
              <span className="ml-auto text-[11.5px] text-faint tabular-nums">
                {unplaced.length || ""}
              </span>
              <Sow where="unset" onSow={onSow} />
              <button
                type="button"
                aria-label={t("hideUnplaced")}
                title={t("hideUnplaced")}
                onClick={() => swing(false)}
                className="grid h-5 w-5 shrink-0 place-items-center rounded text-[12px] leading-none text-faint hover:bg-hover hover:text-soft"
              >
                ✕
              </button>
            </header>
            <div className="px-2.5 pt-2">
              <Only lists={lists} chosen={only} onChange={setOnly} />
            </div>
            {unplaced.length === 0 ? (
              <p className="grid flex-1 place-items-center px-3 text-center text-[12.5px] text-faint">
                {t("allPlaced")}
              </p>
            ) : (
              <ul className="scroller flex-1 p-1.5">{unplaced.map(card)}</ul>
            )}
          </aside>
        )}
      </div>

      {carrying && (
        <div
          ref={ghost}
          aria-hidden
          className="pointer-events-none fixed top-0 left-0 z-50 max-w-72 truncate rounded-md border border-accent bg-bg px-2.5 py-1.5 text-[13px] shadow-lift-tall"
        >
          {carrying.title}
        </div>
      )}
    </section>
  );
}

function Sow({ where, onSow }: { where: Priority; onSow: (where: Priority) => void }) {
  return (
    <button
      type="button"
      aria-label={fill("addTo", said(where))}
      title={fill("addTo", said(where))}
      onClick={() => onSow(where)}
      className="grid h-5 w-5 shrink-0 place-items-center rounded pb-px text-[15px] leading-none text-faint hover:bg-hover hover:text-ink"
    >
      +
    </button>
  );
}

function Axis({ label, down }: { label: string; down?: boolean }) {
  return (
    <span
      className="grid place-items-center text-[10.5px] font-semibold tracking-[0.09em] text-faint uppercase"
      style={down ? { writingMode: "vertical-rl", transform: "rotate(180deg)" } : undefined}
    >
      {label}
    </span>
  );
}
