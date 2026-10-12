import type { Pick } from "../core";
import { fill, t } from "../locales";
import Modal from "./Modal";

interface Props {
  named: string;
  onPick: (pick: Pick | null) => void;
}

const SIDES: { key: Pick; word: "versionsKeepMine" | "versionsKeepTheirs" | "versionsKeepBoth" }[] =
  [
    { key: "mine", word: "versionsKeepMine" },
    { key: "theirs", word: "versionsKeepTheirs" },
    { key: "both", word: "versionsKeepBoth" },
  ];

export default function Versions({ named, onPick }: Props) {
  return (
    <Modal title={fill("riftTitle", named)} onClose={() => onPick(null)}>
      <p className="px-1 text-[13px] leading-relaxed text-soft">{t("versionsWhy")}</p>
      <div className="mt-3 flex flex-col gap-1.5">
        {SIDES.map((side) => (
          <button
            key={side.key}
            type="button"
            onClick={() => onPick(side.key)}
            className="rounded-md border border-line px-3 py-1.5 text-left text-[13px] hover:border-ink"
          >
            {t(side.word)}
          </button>
        ))}
      </div>
      <div className="mt-3 flex justify-end">
        <button
          type="button"
          onClick={() => onPick(null)}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
        >
          {t("versionsLater")}
        </button>
      </div>
    </Modal>
  );
}
