import { useState } from "react";
import type { Filed } from "../core";
import { folded } from "../folding";
import { t } from "../locales";
import Modal from "./Modal";

interface Props {
  idea: string;
  known: Filed[];
  own: string;
  onPick: (file: string) => void;
  onClose: () => void;
}

const MOST = 12;

export default function FileIdea({ idea, known, own, onPick, onClose }: Props) {
  const [query, setQuery] = useState("");
  const asked = folded(query.trim());
  const open = known
    .filter((one) => one.file !== own && !one.away && !one.locked && !one.gone)
    .filter((one) => !asked || folded(one.title || t("untitledDoc")).includes(asked))
    .sort(
      (a, b) =>
        (b.wrote ?? "").localeCompare(a.wrote ?? "") ||
        (a.title || "").localeCompare(b.title || ""),
    )
    .slice(0, MOST);

  return (
    <Modal title={t("ideaFileTitle")} onClose={onClose}>
      <p className="mb-3 text-[12.5px] text-soft">{idea}</p>
      <input
        autoFocus
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && asked && open[0]) onPick(open[0].file);
        }}
        placeholder={t("ideaFileFind")}
        aria-label={t("ideaFileFind")}
        className="w-full rounded-md border border-line bg-bg px-2.5 py-1.5 text-[13px] outline-none focus:border-accent"
      />
      <ul className="mt-2 flex max-h-[18rem] flex-col gap-px overflow-y-auto">
        {open.length === 0 ? (
          <li className="px-2 py-1.5 text-[12.5px] text-faint">{t("ideaFileNone")}</li>
        ) : (
          open.map((one) => (
            <li key={one.file}>
              <button
                type="button"
                onClick={() => onPick(one.file)}
                className="w-full truncate rounded-md px-2 py-1.5 text-left text-[13px] hover:bg-hover"
              >
                {one.title || t("untitledDoc")}
              </button>
            </li>
          ))
        )}
      </ul>
    </Modal>
  );
}
