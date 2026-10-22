import { t } from "../locales";
import type { Stamping } from "./printing";
import { Ask } from "./Rows";

interface Props {
  on: boolean;
  label: string;
  why: string;
  disabled?: boolean;
  /** Sits in a bar beside other controls rather than as a line of a dialog. */
  barred?: boolean;
  onTick: (on: boolean) => void;
}

function PdfTick({ on, label, why, disabled, barred, onTick }: Props) {
  return (
    <div
      className={`flex items-center gap-1.5 ${barred ? "mr-1 text-[12.5px] text-soft" : "mt-3 text-[13px]"}`}
    >
      <label className={`flex items-center ${barred ? "gap-1.5" : "gap-2.5"}`}>
        <input
          type="checkbox"
          checked={on}
          disabled={disabled}
          onChange={(e) => onTick(e.target.checked)}
          className="h-3.5 w-3.5 accent-accent"
        />
        {label}
      </label>
      <Ask said={why} end={barred} />
    </div>
  );
}

interface Ticks {
  stamping: Stamping;
  onStamp: (chosen: Stamping) => void;
  /** A file from outside Tisty has no log to fill a card from. */
  cardable?: boolean;
  disabled?: boolean;
  barred?: boolean;
}

export default function PdfTicks({ stamping, onStamp, cardable = true, disabled, barred }: Ticks) {
  return (
    <>
      {cardable && (
        <PdfTick
          on={stamping.carded}
          label={t("pdfSign")}
          why={t("pdfSignAsk")}
          disabled={disabled}
          barred={barred}
          onTick={(carded) => onStamp({ ...stamping, carded })}
        />
      )}
      <PdfTick
        on={stamping.credited}
        label={t("pdfCreditTick")}
        why={t("pdfCreditAsk")}
        disabled={disabled}
        barred={barred}
        onTick={(credited) => onStamp({ ...stamping, credited })}
      />
    </>
  );
}
