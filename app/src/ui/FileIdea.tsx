import { useState } from "react";
import type { Filed } from "../core";
import { folded } from "../folding";
import { t } from "../locales";
import { tongueName } from "../tongues";
import Modal from "./Modal";

interface Props {
  idea: string;
  known: Filed[];
  own: string;
  onPick: (file: string) => void;
  onClose: () => void;
}

const MOST = 12;

const spoken = (one: Filed): string => (one.tongue && one.lang ? tongueName(one.lang) : "");

// A page carries no language of its own: it is in the language of the document it hangs from.
const whereOf = (one: Filed, byId: ReadonlyMap<string, Filed>): string => {
  const above = one.pageOf ? byId.get(one.pageOf) : undefined;
  let root = one;
  for (let steps = 0; root.pageOf && steps < byId.size; steps++) {
    const up = byId.get(root.pageOf);
    if (!up) break;
    root = up;
  }
  return [above?.title || (above && t("untitledDoc")), spoken(root)].filter(Boolean).join(" · ");
};

export default function FileIdea({ idea, known, own, onPick, onClose }: Props) {
  const [query, setQuery] = useState("");
  const asked = folded(query.trim());
  const byId = new Map(known.map((one) => [one.id, one]));
  const open = known
    .filter((one) => one.file !== own && !one.away && !one.locked && !one.gone)
    .filter(
      (one) =>
        !asked || folded(`${one.title || t("untitledDoc")} ${whereOf(one, byId)}`).includes(asked),
    )
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
                className="flex w-full items-baseline gap-2 rounded-md px-2 py-1.5 text-left text-[13px] hover:bg-hover"
              >
                <span className="min-w-0 truncate">{one.title || t("untitledDoc")}</span>
                {whereOf(one, byId) && (
                  <span className="min-w-0 shrink truncate text-[11.5px] text-faint">
                    {whereOf(one, byId)}
                  </span>
                )}
              </button>
            </li>
          ))
        )}
      </ul>
    </Modal>
  );
}
