import { t, type Word } from "../locales";
import Modal from "./Modal";

export type Door = "merge" | "mine" | "theirs";

interface Props {
  onPick: (door: Door) => void;
  onElse: () => void;
  onClose: () => void;
}

const DOORS: { key: Door; name: Word; why: Word; how: Word }[] = [
  { key: "merge", name: "apartMerge", why: "apartMergeWhy", how: "apartMergeHow" },
  { key: "mine", name: "apartMine", why: "apartMineWhy", how: "apartMineHow" },
  { key: "theirs", name: "apartTheirs", why: "apartTheirsWhy", how: "apartTheirsHow" },
];

export default function Apart({ onPick, onElse, onClose }: Props) {
  return (
    <Modal title={t("apartTitle")} onClose={onClose}>
      <p className="px-1 text-[13px] leading-relaxed text-soft">{t("apartWhy")}</p>

      <ul className="mt-3 flex flex-col gap-2">
        {DOORS.map((door) => (
          <li key={door.key}>
            <button
              type="button"
              onClick={() => onPick(door.key)}
              className="w-full rounded-lg border border-line px-3 py-2.5 text-left hover:border-ink hover:bg-hover"
            >
              <span className="block text-[13.5px] font-semibold">{t(door.name)}</span>
              <span className="mt-0.5 block text-[12px] text-faint">{t(door.why)}</span>
              <span className="mt-1.5 block text-[12px] leading-relaxed whitespace-pre-line text-soft">
                {t(door.how)}
              </span>
            </button>
          </li>
        ))}
      </ul>

      <p className="mt-3 px-1 text-[11.5px] leading-relaxed text-faint">{t("apartUndo")}</p>

      <div className="mt-3 flex justify-end gap-2">
        <button
          type="button"
          onClick={onElse}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-soft hover:bg-hover"
        >
          {t("apartElse")}
        </button>
        <button
          type="button"
          onClick={onClose}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
        >
          {t("close")}
        </button>
      </div>
    </Modal>
  );
}
