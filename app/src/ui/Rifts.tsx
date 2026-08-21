import { useState } from "react";
import type { Pick, Rift } from "../core";
import { fill, t } from "../locales";
import Modal from "./Modal";

interface Props {
  named: string;
  rifts: Rift[];
  onDone: (picks: Pick[]) => void;
  onClose: () => void;
}

const SIDES: { key: Pick; word: "riftKeepMine" | "riftKeepTheirs" | "riftKeepBoth" }[] = [
  { key: "mine", word: "riftKeepMine" },
  { key: "theirs", word: "riftKeepTheirs" },
  { key: "both", word: "riftKeepBoth" },
];

const Said = ({ head, said, dim }: { head: string; said: string[]; dim?: boolean }) => (
  <div className={dim ? "opacity-60" : undefined}>
    <p className="mb-0.5 text-[11px] tracking-[0.04em] text-faint uppercase">{head}</p>
    <pre className="max-h-40 overflow-auto rounded-md bg-hover px-2.5 py-1.5 text-[12px] leading-relaxed whitespace-pre-wrap">
      {said.join("\n\n") || "—"}
    </pre>
  </div>
);

export default function Rifts({ named, rifts, onDone, onClose }: Props) {
  const [picks, setPicks] = useState<(Pick | undefined)[]>(() => rifts.map(() => undefined));
  const left = picks.filter((one) => one === undefined).length;

  return (
    <Modal title={fill("riftTitle", named)} onClose={onClose}>
      <p className="px-1 text-[13px] leading-relaxed text-soft">{t("riftWhy")}</p>

      <ul className="mt-3 flex flex-col gap-3">
        {rifts.map((rift, at) => (
          <li
            key={`${at}:${rift.was.join("")}`}
            className="rounded-lg border border-line px-3 py-2.5"
          >
            {rift.was.length > 0 && <Said head={t("riftWas")} said={rift.was} dim />}
            <div className="mt-2 grid gap-2 sm:grid-cols-2">
              <Said head={t("riftMine")} said={rift.mine} />
              <Said head={t("riftTheirs")} said={rift.theirs} />
            </div>
            <div className="mt-2 flex gap-1.5">
              {SIDES.map((side) => (
                <button
                  key={side.key}
                  type="button"
                  aria-pressed={picks[at] === side.key}
                  onClick={() =>
                    setPicks((was) => was.map((one, n) => (n === at ? side.key : one)))
                  }
                  className={`rounded-md border px-2.5 py-0.5 text-[12px] ${
                    picks[at] === side.key
                      ? "border-ink bg-ink text-bg"
                      : "border-line text-soft hover:border-ink"
                  }`}
                >
                  {t(side.word)}
                </button>
              ))}
            </div>
          </li>
        ))}
      </ul>

      <div className="mt-3 flex items-center justify-end gap-3">
        {left > 0 && (
          <span className="text-[12px] text-faint">{fill("riftLeft", String(left))}</span>
        )}
        <button
          type="button"
          onClick={onClose}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
        >
          {t("close")}
        </button>
        <button
          type="button"
          disabled={left > 0}
          onClick={() => onDone(picks as Pick[])}
          className="rounded-md border border-ink bg-ink px-3 py-1 text-[12.5px] text-bg disabled:border-line disabled:bg-transparent disabled:text-faint"
        >
          {t("riftDone")}
        </button>
      </div>
    </Modal>
  );
}
