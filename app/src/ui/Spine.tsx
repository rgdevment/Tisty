import { useEffect, useRef, useState } from "react";
import type { Task } from "../core";
import Ahead, { HEAVY, spreadOf, weekday } from "./Ahead";

interface Props {
  tasks: Task[];
  days: number;
  onOpen: (task: string) => void;
}

const DOTS = 3;

export default function Spine({ tasks, days, onOpen }: Props) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const spread = spreadOf(tasks, days, new Date());

  useEffect(() => {
    if (!open) return;
    const away = (e: MouseEvent) => {
      if (!box.current?.contains(e.target as Node)) setOpen(false);
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
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
      {open && (
        <div className="shadow-lift-tall absolute inset-x-2 bottom-full z-10 mb-1 max-h-[60vh] overflow-y-auto rounded-[10px] border border-line bg-bg p-2.5">
          <Ahead
            tasks={tasks}
            days={days}
            onOpen={(id) => {
              setOpen(false);
              onOpen(id);
            }}
          />
        </div>
      )}

      <div className="flex items-center gap-0.5">
        {spread.map((day) => (
          <button
            key={day.key}
            type="button"
            aria-expanded={open}
            onClick={() => setOpen(!open)}
            className={`flex flex-1 items-center justify-center gap-1.5 rounded px-1 py-0.5 hover:bg-hover ${
              day.held.length >= HEAVY ? "bg-hue-amber/12" : ""
            }`}
          >
            <span className="text-[9px] font-semibold tracking-[0.05em] text-faint uppercase">
              {weekday().format(day.at)}
            </span>
            <span
              className={`text-[10px] tabular-nums ${
                day.held.length >= HEAVY ? "text-hue-amber" : "text-soft"
              }`}
            >
              {day.at.getDate()}
            </span>
            <span className="flex gap-px" aria-hidden="true">
              {day.held.slice(0, DOTS).map((one) => (
                <span
                  key={one.id}
                  className={`block size-[3px] rounded-full ${
                    one.date?.has_time ? "bg-accent" : "bg-faint/50"
                  }`}
                />
              ))}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
