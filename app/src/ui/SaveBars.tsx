import type { CSSProperties } from "react";
import { fill, t } from "../locales";
import { MILD } from "./Tidied";

export const BAR =
  "mx-auto mb-2 flex w-full flex-wrap items-center gap-x-3 gap-y-1 px-10 text-[11.5px]";

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
  said,
  now,
  onAgain,
}: Readonly<{ style: CSSProperties; said?: string; now?: string; onAgain: () => void }>) {
  return (
    <div role="status" style={style} className={BAR}>
      <span className="text-soft">{said ?? t("saveStuck")}</span>
      <button type="button" onClick={onAgain} className={MILD}>
        {now ?? t("tryAgain")}
      </button>
    </div>
  );
}

// How many places clashed, or `null` when only the whole page could stand; unknown says only that it moved.
const wordsSaid = (torn?: number | null): string | null => {
  if (torn === null) return t("clashWhole");
  if (torn === 1) return t("clashTornOne");
  if (torn !== undefined && torn > 1) return fill("clashTorn", String(torn));
  return null;
};

// Properties both changed are replaced whole by mine, and the bar says so rather than «everything else stays».
export const clashSaid = (torn?: number | null, front = false): string => {
  const words = wordsSaid(torn);
  if (front && (torn === 0 || words === null)) return t("clashFront");
  if (!words) return t("documentMoved");
  return front && torn !== null ? `${words} ${t("clashFrontToo")}` : words;
};

export function ClashBar({
  style,
  torn,
  front,
  onMine,
  onTheirs,
}: Readonly<{
  style: CSSProperties;
  torn?: number | null;
  front?: boolean;
  onMine: () => void;
  onTheirs: () => void;
}>) {
  return (
    <div role="status" style={style} className={BAR}>
      <span className="text-soft">{clashSaid(torn, front)}</span>
      <button type="button" onClick={onMine} className={MILD}>
        {t("clashSave")}
      </button>
      <button type="button" onClick={onTheirs} className={MILD}>
        {t("clashLook")}
      </button>
    </div>
  );
}
