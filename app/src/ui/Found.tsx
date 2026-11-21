import { fill, t } from "../locales";
import { found, type Spot } from "../settingsRows";

interface Props {
  asked: string;
  onLand: (spot: Spot) => void;
}

export default function Found({ asked, onLand }: Props) {
  const hits = found(asked);
  return (
    <>
      <h2 className="text-[21px] font-semibold">«{asked.trim()}»</h2>
      <p role="status" className="mt-1 text-[13px] text-soft">
        {hits.length === 1 ? t("findOne") : fill("findMany", String(hits.length))}
      </p>
      {hits.length > 0 ? (
        <div className="mt-2.5 overflow-hidden rounded-[10px] border border-hair">
          {hits.map((hit) => (
            <button
              key={hit.id}
              type="button"
              onClick={() => onLand({ page: hit.row.page, shelf: hit.row.shelf, row: hit.id })}
              className="flex w-full items-center gap-2.5 border-t border-hair px-3 py-2 text-left first:border-t-0 hover:bg-hover"
            >
              <span className="min-w-0 flex-1 text-[13px] font-medium">{hit.title}</span>
              <span className="shrink-0 text-[11.5px] whitespace-nowrap text-accent">
                {hit.page} ›
              </span>
            </button>
          ))}
        </div>
      ) : (
        <p className="mt-2 text-[11.5px] text-faint">{t("findNone")}</p>
      )}
    </>
  );
}
