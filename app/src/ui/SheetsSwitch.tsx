import { t } from "../locales";
import Glyph from "./Glyph";

export function SheetsToggle({
  sheets,
  onSheets,
  className,
}: Readonly<{ sheets: boolean; onSheets: (on: boolean) => void; className: string }>) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={sheets}
      aria-label={t("sheetsShown")}
      title={t("sheetsShown")}
      onClick={() => onSheets(!sheets)}
      className={`${className} w-7 justify-center ${
        sheets ? "border-accent/40 bg-active text-ink" : "hover:bg-hover"
      }`}
    >
      <Glyph name="pagebreak" className={`h-3.5 w-3.5 ${sheets ? "text-accent" : ""}`} />
    </button>
  );
}

export function SheetsTurn({
  landscape,
  onLandscape,
  className,
}: Readonly<{ landscape: boolean; onLandscape: (across: boolean) => void; className: string }>) {
  return (
    <div className={`flex justify-end gap-3 text-[11.5px] text-faint ${className}`}>
      {[false, true].map((turned) => (
        <button
          key={String(turned)}
          type="button"
          aria-pressed={landscape === turned}
          onClick={() => onLandscape(turned)}
          className={landscape === turned ? "text-ink" : "hover:text-soft"}
        >
          {t(turned ? "sheetsAcross" : "sheetsUpright")}
        </button>
      ))}
    </div>
  );
}
