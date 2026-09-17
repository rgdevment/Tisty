import { useEffect, useRef, useState } from "react";
import { AXES, type Axis as Grouping } from "../archive";
import { t } from "../locales";
import { axisWord } from "../views";

interface Props {
  axis: Grouping;
  onChange: (axis: Grouping) => void;
}

const PILL = { time: "byTime", list: "byList", tag: "byTag", quadrant: "byKind" } as const;

export default function Axis({ axis, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const box = useRef<HTMLDivElement>(null);

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

  const grouped = axis !== "time";

  return (
    <div ref={box} className="relative">
      <button
        type="button"
        aria-expanded={open}
        aria-haspopup="true"
        aria-label={t("archiveGrouped")}
        onClick={() => setOpen((was) => !was)}
        className={`flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-[11.5px] ${
          grouped ? "border-ink bg-ink text-bg" : "border-line text-faint hover:text-soft"
        }`}
      >
        {t(PILL[axis])}
        <span aria-hidden="true" className="opacity-70">
          ▾
        </span>
      </button>

      {open && (
        <fieldset className="absolute left-0 z-30 mt-1 w-48 rounded-[10px] border border-hair bg-bg p-1.5 shadow-lift">
          <legend className="sr-only">{t("archiveGrouped")}</legend>
          {AXES.map((one) => (
            <label
              key={one}
              className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-[12.5px] text-soft hover:bg-hover"
            >
              <input
                type="radio"
                name="axis"
                checked={axis === one}
                onChange={() => {
                  onChange(one);
                  setOpen(false);
                }}
              />
              <span className="min-w-0 flex-1 truncate">{t(axisWord(one))}</span>
            </label>
          ))}
        </fieldset>
      )}
    </div>
  );
}
