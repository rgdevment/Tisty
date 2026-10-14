import type { Kin } from "../core";
import { fill, t, type Word } from "../locales";
import Modal from "./Modal";

export type Door = "merge" | "mine" | "theirs";

interface Props {
  kin: Kin;
  cloud?: string;
  onPick: (door: Door) => void;
  onElse: () => void;
  onClose: () => void;
}

const DOORS: { key: Door; name: Word; why: Word; how: Word }[] = [
  { key: "merge", name: "apartMerge", why: "apartMergeWhy", how: "apartMergeHow" },
  { key: "mine", name: "apartMine", why: "apartMineWhy", how: "apartMineHow" },
  { key: "theirs", name: "apartTheirs", why: "apartTheirsWhy", how: "apartTheirsHow" },
];

const CLOUDED: Partial<Record<Word, Word>> = {
  apartTitle: "apartTitleCloud",
  apartHomeTitle: "apartHomeTitleCloud",
  apartHomeWhy: "apartHomeWhyCloud",
  apartHomeDo: "apartHomeDoCloud",
  apartUnsureTitle: "apartUnsureTitleCloud",
  apartUnsureWhy: "apartUnsureWhyCloud",
  apartMineWhy: "apartMineWhyCloud",
  apartMineHow: "apartMineHowCloud",
  apartTheirs: "apartTheirsCloud",
  apartTheirsHow: "apartTheirsHowCloud",
};

const placed = (key: Word, cloud?: string): string => {
  const clouded = CLOUDED[key];
  return cloud && clouded ? fill(clouded, cloud) : t(key);
};

export default function Apart({ kin, cloud, onPick, onElse, onClose }: Props) {
  const say = (key: Word) => placed(key, cloud);
  if (kin === "unsure") {
    return (
      <Modal title={say("apartUnsureTitle")} onClose={onClose}>
        <p className="px-1 text-[13px] leading-relaxed text-soft">{say("apartUnsureWhy")}</p>
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
      <Modal title={say("apartHomeTitle")} onClose={onClose}>
        <p className="px-1 text-[13px] leading-relaxed text-soft">{say("apartHomeWhy")}</p>

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
            {say("apartHomeDo")}
          </button>
        </div>
      </Modal>
    );
  }

  const shown = kin === "clash" ? DOORS.filter((one) => one.key !== "merge") : DOORS;

  return (
    <Modal title={say("apartTitle")} onClose={onClose}>
      <p className="px-1 text-[13px] leading-relaxed text-soft">
        {t(kin === "clash" ? "apartClashWhy" : "apartWhy")}
      </p>

      <ul className="mt-3 flex flex-col gap-2">
        {shown.map((door) => (
          <li key={door.key}>
            <button
              type="button"
              onClick={() => onPick(door.key)}
              className="w-full rounded-[10px] border border-line px-3 py-2.5 text-left hover:border-ink hover:bg-hover"
            >
              <span className="block text-[13px] font-semibold">{say(door.name)}</span>
              <span className="mt-0.5 block text-[12.5px] text-faint">{say(door.why)}</span>
              <span className="mt-1.5 block text-[12.5px] leading-relaxed whitespace-pre-line text-soft">
                {say(door.how)}
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
