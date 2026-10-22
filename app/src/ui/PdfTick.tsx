import { Ask } from "./Rows";

interface Props {
  on: boolean;
  label: string;
  why?: string;
  disabled?: boolean;
  /** Sits in a bar beside other controls rather than as a line of a dialog. */
  barred?: boolean;
  onTick: (on: boolean) => void;
}

export default function PdfTick({ on, label, why, disabled, barred, onTick }: Props) {
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
      {why && <Ask said={why} end={barred} />}
    </div>
  );
}
