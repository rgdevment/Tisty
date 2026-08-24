import { t } from "../locales";

interface Props {
  at: { x: number; y: number };
  onOpen: () => void;
  onKeep?: () => void;
  onDrop: () => void;
}

export default function Shot({ at, onOpen, onKeep, onDrop }: Props) {
  const left = Math.max(8, Math.min(at.x, window.innerWidth - (onKeep ? 330 : 190)));
  const top = Math.max(8, at.y);

  return (
    <div
      style={{ left, top }}
      className="fixed z-40 flex items-center gap-0.5 rounded-[10px] border border-hair bg-rail p-1 shadow-xl"
    >
      <button
        type="button"
        onMouseDown={(e) => e.preventDefault()}
        onClick={onOpen}
        className="rounded-md px-2 py-1 text-[12px] text-soft hover:bg-hover hover:text-ink"
      >
        {t("seeWhole")}
      </button>
      {onKeep && (
        <button
          type="button"
          onMouseDown={(e) => e.preventDefault()}
          onClick={onKeep}
          className="rounded-md px-2 py-1 text-[12px] text-soft hover:bg-hover hover:text-ink"
        >
          {t("keepACopy")}
        </button>
      )}
      <span className="h-4 w-px bg-hair" />
      <button
        type="button"
        onMouseDown={(e) => e.preventDefault()}
        onClick={onDrop}
        className="rounded-md px-2 py-1 text-[12px] text-soft hover:bg-hover hover:text-urgent"
      >
        {t("remove")}
      </button>
    </div>
  );
}
