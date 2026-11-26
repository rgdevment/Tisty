import { undo, useGoing } from "../dropLater";
import { fill, t } from "../locales";

/// What was just deleted waits a few seconds here, so a slip of the hand costs one click.
export default function DropUndo() {
  const going = useGoing();
  const last = going[going.length - 1];
  if (!last) return null;
  return (
    <div className="cheer">
      <output className="flex items-center gap-3 rounded-[10px] border border-line bg-bg p-[12px_14px] shadow-lift">
        <span
          aria-hidden="true"
          className="grid size-[30px] flex-none place-items-center rounded-full bg-hover text-[13px] text-urgent"
        >
          ✕
        </span>
        <span className="min-w-0 flex-1">
          <span title={last.said} className="block truncate text-[13px] font-semibold">
            {last.said}
          </span>
          {going.length > 1 && (
            <span className="block text-[11.5px] text-faint">
              {fill("dropsWaiting", String(going.length))}
            </span>
          )}
        </span>
        <button
          type="button"
          onClick={() => undo(last.key)}
          aria-label={fill("dropUndoWhich", last.said)}
          className="rounded-md px-2 py-1 text-[12.5px] text-accent hover:bg-hover"
        >
          {t("cheerUndo")}
        </button>
      </output>
    </div>
  );
}
