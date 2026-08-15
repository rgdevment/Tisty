import type { Kin } from "../core";
import { t, type Word } from "../locales";
import Modal from "./Modal";

export type Door = "merge" | "mine" | "theirs";

interface Props {
  kin: Kin;
  onPick: (door: Door) => void;
  onElse: () => void;
  onClose: () => void;
}

const DOORS: { key: Door; name: Word; why: Word; how: Word }[] = [
  { key: "merge", name: "apartMerge", why: "apartMergeWhy", how: "apartMergeHow" },
  { key: "mine", name: "apartMine", why: "apartMineWhy", how: "apartMineHow" },
  { key: "theirs", name: "apartTheirs", why: "apartTheirsWhy", how: "apartTheirsHow" },
];

export default function Apart({ kin, onPick, onElse, onClose }: Props) {
  if (kin === "unsure") {
    return (
      <Modal title={t("apartUnsureTitle")} onClose={onClose}>
        <p className="px-1 text-[13px] leading-relaxed text-soft">{t("apartUnsureWhy")}</p>
        <div className="mt-4 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="rounded-md border border-line px-3 py-1 text-[12.5px] hover:bg-hover"
          >
            {t("close")}
          </button>
        </div>
      </Modal>
    );
  }

  if (kin === "sameLineage") {
    return (
      <Modal title={t("apartHomeTitle")} onClose={onClose}>
        <p className="px-1 text-[13px] leading-relaxed text-soft">{t("apartHomeWhy")}</p>

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
          >
            {t("close")}
          </button>
          <button
            type="button"
            onClick={() => onPick("merge")}
            className="rounded-md border border-ink bg-ink px-3 py-1 text-[12.5px] text-bg"
          >
            {t("apartHomeDo")}
          </button>
        </div>
      </Modal>
    );
  }

  const shown = kin === "clash" ? DOORS.filter((one) => one.key !== "merge") : DOORS;

  return (
    <Modal title={t("apartTitle")} onClose={onClose}>
      <p className="px-1 text-[13px] leading-relaxed text-soft">
        {t(kin === "clash" ? "apartClashWhy" : "apartWhy")}
      </p>

      <ul className="mt-3 flex flex-col gap-2">
        {shown.map((door) => (
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
