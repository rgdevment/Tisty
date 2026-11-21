import type { CSSProperties } from "react";
import { t } from "../locales";
import { MILD } from "./Tidied";

const BAR = "mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]";

export function SaveLine({
  style,
  saving,
  said,
}: Readonly<{ style: CSSProperties; saving: boolean; said: string }>) {
  return (
    <div
      aria-live="polite"
      style={style}
      className="mx-auto h-5 w-full px-10 text-[11.5px] text-faint"
    >
      {saving ? <span className="motion-safe:animate-pulse">{t("saving")}</span> : said}
    </div>
  );
}

export function StuckBar({
  style,
  onAgain,
}: Readonly<{ style: CSSProperties; onAgain: () => void }>) {
  return (
    <div role="status" style={style} className={BAR}>
      <span className="text-soft">{t("saveStuck")}</span>
      <button type="button" onClick={onAgain} className={MILD}>
        {t("tryAgain")}
      </button>
    </div>
  );
}

export function ClashBar({
  style,
  onMine,
  onTheirs,
}: Readonly<{ style: CSSProperties; onMine: () => void; onTheirs: () => void }>) {
  return (
    <div style={style} className={BAR}>
      <span className="text-soft">{t("documentMoved")}</span>
      <button type="button" onClick={onMine} className={MILD}>
        {t("clashSave")}
      </button>
      <button type="button" onClick={onTheirs} className={MILD}>
        {t("clashLook")}
      </button>
    </div>
  );
}
