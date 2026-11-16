import { t } from "../locales";
import Glyph from "./Glyph";

interface Props {
  over: string;
  big: string;
  open: boolean | null;
  onTurn: () => void;
  onEdit?: () => void;
}

export default function DiaryHead({ over, big, open, onTurn, onEdit }: Props) {
  return (
    <header className="flex items-end justify-between gap-4">
      <div className="min-w-0">
        {over && <p className="text-[11.5px] text-faint">{over}</p>}
        <h1 className="truncate text-[21px] font-semibold text-ink">{big}</h1>
      </div>
      <div className="flex shrink-0 items-center gap-1">
        {open !== null && (
          <button
            type="button"
            aria-pressed={open}
            onClick={onTurn}
            title={t(open ? "diaryOpenWhy" : "diaryHiddenWhy")}
            className={`flex items-center gap-1.5 rounded-md px-2 py-1 text-[11.5px] hover:bg-hover ${
              open ? "text-accent" : "text-soft"
            }`}
          >
            <Glyph name={open ? "unlocked" : "lock"} className="h-[13px] w-[13px]" />
            {t(open ? "diaryOpen" : "diaryHidden")}
          </button>
        )}
        {onEdit && (
          <button
            type="button"
            onClick={onEdit}
            aria-label={t("diaryEditMonth")}
            title={t("diaryEditMonth")}
            className="grid size-7 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
          >
            <Glyph name="pencil" className="h-[14px] w-[14px]" />
          </button>
        )}
      </div>
    </header>
  );
}
