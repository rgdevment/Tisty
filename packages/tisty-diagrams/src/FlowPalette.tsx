import { outlineOf } from "./flowoutlines";
import type { Shape } from "./flowshapes";
import { useSay } from "./handing";
import { PALETTE, shapeName } from "./paletteshapes";

/** What a shape dragged from the palette carries, so the canvas knows it from any other drop. */
export const DRAGGED = "application/x-tisty-shape";

function Preview({ shape }: Readonly<{ shape: Shape }>) {
  const outline = outlineOf(shape, 34, 22);
  return (
    <svg
      aria-hidden="true"
      width="36"
      height="24"
      viewBox="-1 -1 36 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.3"
    >
      {outline.body && <path d={outline.body} />}
      {outline.lines.map((d) => (
        <path key={d} d={d} />
      ))}
      {!outline.body && !outline.lines.length && <path d="M8,7 H26 M8,12 H26 M8,17 H19" />}
    </svg>
  );
}

/** The shapes to drag onto the canvas, or to press and find in the middle of it. */
export default function FlowPalette({ onAdd }: Readonly<{ onAdd: (shape: Shape) => void }>) {
  const t = useSay();
  return (
    <aside
      aria-label={t("flowPalette")}
      className="scroller flex w-[212px] shrink-0 flex-col gap-3 overflow-y-auto border-r border-hair bg-rail p-3"
    >
      {PALETTE.map((group) => (
        <section key={group.title} aria-label={t(group.title)} className="flex flex-col gap-1">
          <h3 className="m-0 text-[11.5px] font-semibold text-faint">{t(group.title)}</h3>
          <div className="grid grid-cols-3 gap-1">
            {group.shapes.map((shape) => (
              <button
                key={shape}
                type="button"
                draggable
                onDragStart={(e) => {
                  e.dataTransfer.setData(DRAGGED, shape);
                  e.dataTransfer.effectAllowed = "copy";
                }}
                onClick={() => onAdd(shape)}
                className="flex h-[58px] cursor-grab flex-col items-center justify-center gap-1 rounded-md text-[10.5px] leading-tight text-soft hover:bg-hover hover:text-ink"
              >
                <Preview shape={shape} />
                {shapeName(shape, t)}
              </button>
            ))}
          </div>
        </section>
      ))}
      <p className="m-0 mt-auto text-[11.5px] leading-snug text-faint">{t("flowPaletteHint")}</p>
    </aside>
  );
}
