import { useEffect, useState } from "react";
import { renameMachine, type ThisMachine, thisMachine } from "../core";
import { fill, t } from "../locales";
import type { Which, Word } from "./Card";
import Modal from "./Modal";
import { mild, strong } from "./Rows";

const NAMED_AT_MOST = 64;

type Run = <T>(card: Which, work: Promise<T>, then: (answer: T) => void) => void;

interface Props {
  busy: Which | null;
  run: Run;
  tell: (word?: Word) => void;
}

export default function Here({ busy, run, tell }: Props) {
  const held = busy !== null;
  const [here, setHere] = useState<ThisMachine | null>(null);
  const [big, setBig] = useState(false);
  const [naming, setNaming] = useState<string | null>(null);

  useEffect(() => {
    thisMachine()
      .then(setHere)
      .catch(() => {});
  }, []);

  if (!here) return null;
  const name = here.name ?? t("machineHere");

  const save = () => {
    if (naming === null) return;
    run("sync", renameMachine(naming), (now) => {
      setHere(now);
      setNaming(null);
      tell({ card: "sync", text: fill("renamed", now.name ?? t("machineHere")) });
    });
  };

  return (
    <div className="mt-2.5 border-t border-hair pt-2.5">
      <p className="text-[12.5px] text-soft">{fill("thisMachineIs", name)}</p>
      {here.code && (
        <p className="mt-1 text-[12.5px] text-soft" title={t("thisMachineCodeWhy")}>
          {t("thisMachineCode")} <span className="font-mono text-ink">{here.code}</span>
        </p>
      )}
      {naming === null ? (
        <div className="mt-2 flex flex-wrap items-center gap-2.5">
          {here.code && (
            <button type="button" onClick={() => setBig(true)} className={mild}>
              {t("showBig")}
            </button>
          )}
          <button
            type="button"
            disabled={held}
            onClick={() => setNaming(here.name ?? "")}
            className={mild}
          >
            {t("renameMachine")}
          </button>
        </div>
      ) : (
        <form
          className="mt-2"
          onSubmit={(event) => {
            event.preventDefault();
            save();
          }}
        >
          <input
            aria-label={t("renameMachine")}
            value={naming}
            maxLength={NAMED_AT_MOST}
            onChange={(event) => setNaming(event.target.value)}
            className="w-full rounded-[10px] border border-line bg-bg px-2.5 py-1 text-[12.5px]"
          />
          <p className="mt-1 text-[11.5px] text-faint">{t("renameMachineWhat")}</p>
          <div className="mt-2 flex flex-wrap items-center gap-2.5">
            <button type="submit" disabled={held} className={strong}>
              {t("saveIt")}
            </button>
            <button type="button" onClick={() => setNaming(null)} className={mild}>
              {t("cancel")}
            </button>
          </div>
        </form>
      )}
      {big && here.code && (
        <Modal title={fill("showBigTitle", name)} wide onClose={() => setBig(false)}>
          <p className="text-[13px] leading-relaxed text-soft">{t("showBigWhat")}</p>
          <p className="mt-4 text-center font-mono text-[34px] tracking-[0.08em] [word-spacing:0.4em] tabular-nums">
            {here.code}
          </p>
          <div className="mt-5 flex justify-end">
            <button type="button" onClick={() => setBig(false)} className={mild}>
              {t("close")}
            </button>
          </div>
        </Modal>
      )}
    </div>
  );
}
