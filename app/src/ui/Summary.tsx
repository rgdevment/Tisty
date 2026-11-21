import { type Dot, type Item, type Step, stepsShown, TILE_PAGES, type Tile } from "../attention";
import { fill, t } from "../locales";
import { pageOf } from "../settingsPages";
import type { Spot } from "../settingsRows";
import Glyph from "./Glyph";
import { Band } from "./Rows";

interface Props {
  items: Item[] | null;
  whole: boolean;
  steps: Step[] | null;
  tiles: Record<string, Tile | null>;
  onLand: (spot: Spot) => void;
}

const DOT: Record<Dot, string> = {
  ok: "bg-hue-green",
  warn: "bg-hue-amber",
  off: "bg-line",
};

const box = "mt-2.5 overflow-hidden rounded-[10px] border border-hair";
const line =
  "flex w-full items-center gap-2.5 border-t border-hair px-3 py-2 text-left first:border-t-0 hover:bg-hover";

export default function Summary({ items, whole, steps, tiles, onLand }: Props) {
  const first = stepsShown(steps);
  return (
    <>
      {first && steps && <Steps steps={steps} onLand={onLand} />}
      {items && items.length > 0 && (
        <>
          <Band label={fill("toAttendMany", String(items.length))} />
          <div className={box}>
            {items.map((one) => (
              <button key={one.key} type="button" onClick={() => onLand(one.spot)} className={line}>
                <span
                  aria-hidden="true"
                  className="size-[7px] shrink-0 rounded-full bg-hue-amber"
                />
                <span className="min-w-0 flex-1 text-[13px]">{one.said}</span>
                <span className="shrink-0 text-[11.5px] whitespace-nowrap text-accent">
                  {one.act} ›
                </span>
              </button>
            ))}
          </div>
        </>
      )}
      {items?.length === 0 && whole && !first && (
        <>
          <Band label={t("toAttend")} />
          <p className="flex items-center gap-1.5 text-[11.5px] text-faint">
            <span aria-hidden="true" className="size-[7px] rounded-full bg-hue-green" />
            {t("nothingAsks")}
          </p>
        </>
      )}
      <Band label={t("standingBand")} />
      <div className="mt-2.5 grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-2.5">
        {TILE_PAGES.map((page) => (
          <Tiled key={page} page={page} tile={tiles[page]} onLand={onLand} />
        ))}
      </div>
    </>
  );
}

function Steps({ steps, onLand }: { steps: Step[]; onLand: (spot: Spot) => void }) {
  const done = steps.filter((one) => one.done).length;
  return (
    <>
      <Band label={fill("firstSteps", String(done))} />
      <div className={box}>
        {steps.map((one, at) => (
          <button key={one.key} type="button" onClick={() => onLand(one.spot)} className={line}>
            <span
              className={`grid size-5 shrink-0 place-items-center rounded-full border-[1.5px] text-[10.5px] ${
                one.done ? "border-hue-green text-hue-green" : "border-line text-faint"
              }`}
            >
              {one.done ? (
                <>
                  <span aria-hidden="true">✓</span>
                  <span className="sr-only">{t("firstDone")}</span>
                </>
              ) : (
                at + 1
              )}
            </span>
            <span className="min-w-0 flex-1">
              <span
                className={`block text-[13px] font-medium ${one.done ? "text-faint line-through" : ""}`}
              >
                {one.said}
              </span>
              <span className="block text-[12.5px] text-faint">{one.why}</span>
            </span>
            <span className="shrink-0 text-[11.5px] whitespace-nowrap text-accent">
              {t("firstGo")} ›
            </span>
          </button>
        ))}
      </div>
    </>
  );
}

function Tiled({
  page,
  tile,
  onLand,
}: {
  page: Tile["page"];
  tile: Tile | null;
  onLand: (spot: Spot) => void;
}) {
  const shown = pageOf(page);
  return (
    <button
      type="button"
      aria-busy={tile === null}
      onClick={() => onLand({ page })}
      className="block rounded-[10px] border border-hair bg-bg px-3 py-3 text-left hover:bg-hover"
    >
      <span className="flex items-center gap-2 text-[13px] font-semibold">
        <span className="text-soft">
          <Glyph name={shown.icon} />
        </span>
        {t(shown.label)}
      </span>
      <span className="mt-1.5 flex items-center gap-1.5 text-[12.5px]">
        <span
          aria-hidden="true"
          className={`size-[7px] shrink-0 rounded-full ${DOT[tile?.dot ?? "off"]}`}
        />
        <span className={tile ? "" : "text-faint"}>{tile?.head ?? "…"}</span>
      </span>
      <span className="mt-0.5 block min-h-[1lh] text-[11.5px] text-faint">{tile?.detail}</span>
    </button>
  );
}
