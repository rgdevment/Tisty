import { useEffect, useRef, useState } from "react";
import type { Coming, Habit } from "../core";
import Ahead, { HEAVY, specOf, spreadOf, weekday } from "./Ahead";

interface Props {
  coming: Coming[];
  routines: Habit[];
  days: number;
  onOpen: (task: string) => void;
}

const DOTS = 3;

export default function Spine({ coming, routines, days, onOpen }: Props) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const from = useRef<HTMLButtonElement | null>(null);
  const spread = spreadOf(coming, days, new Date());

  useEffect(() => {
    if (!open) return;
    const away = (e: MouseEvent) => {
      if (!box.current?.contains(e.target as Node)) setOpen(false);
    };
    const key = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      setOpen(false);
      from.current?.focus();
    };
    document.addEventListener("mousedown", away);
    document.addEventListener("keydown", key);
    return () => {
      document.removeEventListener("mousedown", away);
      document.removeEventListener("keydown", key);
    };
  }, [open]);

  return (
    <div ref={box} className="relative h-7 shrink-0 border-t border-hair bg-panel px-2 py-1">
      <div className="flex items-center gap-0.5">
        {spread.map((day) => (
          <button
            key={day.key}
            type="button"
            aria-expanded={open}
            aria-controls="spine-days"
            onClick={(e) => {
              from.current = e.currentTarget;
              setOpen(!open);
            }}
            className={`flex flex-1 items-center justify-center gap-1.5 rounded-md px-1 py-0.5 hover:bg-hover ${
              day.held.length >= HEAVY ? "bg-hue-amber/10" : ""
            }`}
          >
            <span className="text-[9px] font-semibold tracking-[0.05em] text-faint uppercase">
              {weekday().format(day.at)}
            </span>
            <span
              className={`text-[10.5px] tabular-nums ${
                day.held.length >= HEAVY ? "text-hue-amber" : "text-soft"
              }`}
            >
              {day.at.getDate()}
            </span>
            <span className="flex gap-px" aria-hidden="true">
              {day.held.slice(0, DOTS).map((one) => (
                <span
                  key={`${one.task.id} ${one.on}`}
                  className={`block size-[3px] rounded-full ${
                    specOf(one)?.has_time ? "bg-accent" : "bg-faint/40"
                  }`}
                />
              ))}
            </span>
          </button>
        ))}
      </div>

      {open && (
        <div
          id="spine-days"
          className="shadow-lift-tall absolute inset-x-2 bottom-full z-10 mb-1 max-h-[60vh] overflow-y-auto rounded-[10px] border border-line bg-bg p-2.5"
        >
          <Ahead
            coming={coming}
            routines={routines}
            days={days}
            onOpen={(id) => {
              setOpen(false);
              onOpen(id);
            }}
          />
        </div>
      )}
    </div>
  );
}
