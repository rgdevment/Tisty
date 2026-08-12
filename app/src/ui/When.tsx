import { useState } from "react";
import { t } from "../locales";
import Calendar from "./Calendar";

interface Props {
  value?: string;
  clock?: string;
  never?: boolean;
  /// Names the button that applies it. Without one, touching a day applies
  /// straight away — fine for a due date, wrong for a reminder, where the hour
  /// is the point and gets typed after the day.
  confirm?: string;
  onPick: (at: string) => void;
  onClear: () => void;
  onClose: () => void;
}

export default function When({ value, clock, never, confirm, onPick, onClear, onClose }: Props) {
  const [at, setAt] = useState(clock ?? "");
  const [day, setDay] = useState(value ?? "");
  const said = (iso: string) => onPick(at ? `${iso}T${at}:00` : iso);
  /// An hour is typed digit by digit, so nothing in it says «that is the one I
  /// meant» — without a button, changing 10:00 to 09:00 could not be applied at
  /// all. A bare day still lands on the touch, which is what makes it quick.
  const asks = confirm ?? (at ? t("setWhen") : undefined);

  return (
    <>
      <div className="mb-1 flex items-center gap-1.5">
        <span className="w-4 text-center text-[13px] text-faint">⏱</span>
        <input
          type="time"
          value={at}
          aria-label={t("atWhatTime")}
          onChange={(e) => setAt(e.target.value)}
          className="min-w-0 flex-1 rounded-md bg-hover px-2.5 py-1.5 outline-none"
        />
        {at && (
          <button
            type="button"
            aria-label={t("allDay")}
            title={t("allDay")}
            onClick={() => setAt("")}
            className="flex h-6 w-6 items-center justify-center rounded text-faint hover:bg-hover hover:text-ink"
          >
            ×
          </button>
        )}
      </div>
      <Calendar
        inline
        value={asks ? day : value}
        onPick={(iso) => (asks ? setDay(iso) : said(iso))}
        onClear={never ? onClose : onClear}
        onClose={onClose}
      />

      {asks && (
        <div className="mt-1.5 flex items-center gap-1.5">
          <button
            type="button"
            disabled={!day}
            onClick={() => said(day)}
            className="flex-1 rounded-md bg-accent px-2.5 py-1.5 text-bg disabled:opacity-40"
          >
            {asks}
          </button>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md px-2.5 py-1.5 text-faint hover:bg-hover hover:text-ink"
          >
            {t("cancel")}
          </button>
        </div>
      )}
    </>
  );
}
