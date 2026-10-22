import { t } from "../locales";
import { Ask } from "./Rows";

interface Props {
  on: boolean;
  disabled?: boolean;
  /** Sits in a bar beside other controls rather than as a line of a dialog. */
  barred?: boolean;
  onTick: (on: boolean) => void;
}

export default function CreditTick({ on, disabled, barred, onTick }: Props) {
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
        {t("pdfCreditTick")}
      </label>
      <Ask said={t("pdfCreditAsk")} end={barred} />
    </div>
  );
}
