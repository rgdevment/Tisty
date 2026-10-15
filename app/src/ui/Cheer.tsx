import { useEffect, useRef } from "react";
import type { Task } from "../core";
import { fill, t } from "../locales";
import { Head, Thumb } from "./Beaver";

interface Props {
  readonly task: Task;
  readonly onUndo: () => void;
  readonly onDismiss: () => void;
}

const LINGERS = 5000;

export default function Cheer({ task, onUndo, onDismiss }: Props) {
  const leave = useRef(onDismiss);
  leave.current = onDismiss;
  useEffect(() => {
    const timer = setTimeout(() => leave.current(), LINGERS);
    return () => clearTimeout(timer);
  }, []);

  return (
    <div className="cheer">
      <Head className="cheer-head" />
      <Thumb className="cheer-thumb" />
      <div
        role="status"
        className="flex items-center gap-3 rounded-[10px] border border-line bg-bg p-[12px_14px] shadow-lift"
      >
        <span
          aria-hidden="true"
          className="grid size-[30px] flex-none place-items-center rounded-full bg-pen-green text-[13px] text-hue-green"
        >
          ✓
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-[13px] font-semibold">{t("done")}</span>
          <span className="block truncate text-[11.5px] text-faint">
            {fill("cheerKept", task.title)}
          </span>
        </span>
        <button
          type="button"
          onClick={onUndo}
          className="rounded-md px-2 py-1 text-[12.5px] text-accent hover:bg-hover"
        >
          {t("cheerUndo")}
        </button>
      </div>
    </div>
  );
}
