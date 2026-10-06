import { useEffect, useState } from "react";
import { confirmMachineKey, type Machine, syncNow, type ThisMachine, thisMachine } from "../core";
import { done, knock, shownAs } from "../knocking";
import { fill, locale, t } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";

type Step = "compare" | "bringing" | "brought" | "mismatch";

const startedAt = (seconds: number | undefined): string | null =>
  seconds
    ? new Date(seconds * 1000).toLocaleString(locale(), {
        dateStyle: "medium",
        timeStyle: "short",
      })
    : null;

export default function Confirming({ waiting: asked }: { waiting: Machine[] }) {
  const [waiting] = useState(asked);
  const [at, setAt] = useState(0);
  const [step, setStep] = useState<Step>("compare");
  const [why, setWhy] = useState(false);
  const [here, setHere] = useState<ThisMachine | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const one = waiting[at] ?? null;
  const next = waiting[at + 1] ?? null;

  useEffect(() => {
    void thisMachine()
      .then(setHere)
      .catch(() => setHere(null));
  }, []);

  if (!one) return null;
  const name = shownAs(one);

  const yes = () => {
    if (!one.signs) return;
    setTrouble(null);
    setStep("bringing");
    confirmMachineKey(one.id, one.signs)
      .then(() => syncNow("pull"))
      .then(() => setStep("brought"))
      .catch((e) => {
        setTrouble(saidPlainly(e));
        setStep("compare");
      })
      .finally(() => void knock());
  };

  const onward = () => {
    setAt((was) => was + 1);
    setStep("compare");
    setWhy(false);
  };

  if (step === "mismatch") {
    return (
      <Modal title={t("confirmMismatch")} wide onClose={done}>
        <p className="text-[13px] leading-relaxed text-soft">{t("confirmMismatchWhat")}</p>
        <p className="mt-3 rounded-[10px] border border-hair bg-panel px-3.5 py-3 text-[12.5px] leading-relaxed">
          {t("confirmMismatchDo")}
        </p>
        <div className="mt-5 flex justify-end">
          <button
            type="button"
            onClick={done}
            className="rounded-[10px] bg-accent px-3.5 py-1.5 text-[12.5px] font-semibold text-bg"
          >
            {t("confirmGotIt")}
          </button>
        </div>
      </Modal>
    );
  }

  if (step === "bringing" || step === "brought") {
    return (
      <Modal title={fill("confirmDone", name)} wide onClose={done}>
        <p role="status" aria-live="polite" className="text-[13px] text-soft">
          {t(step === "bringing" ? "confirmBringing" : "confirmBrought")}
        </p>
        <div className="mt-5 flex justify-end gap-2 text-[12.5px]">
          <button
            type="button"
            onClick={done}
            className="rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
          >
            {t("confirmClose")}
          </button>
          {next && (
            <button
              type="button"
              onClick={onward}
              className="rounded-[10px] bg-accent px-3.5 py-1.5 font-semibold text-bg"
            >
              {fill("confirmNext", shownAs(next))}
            </button>
          )}
        </div>
      </Modal>
    );
  }

  const since = startedAt(one.since);
  return (
    <Modal title={fill("confirmTitle", name)} wide onClose={done}>
      {waiting.length > 1 && (
        <p className="-mt-1 mb-2 text-[11.5px] text-faint">
          {fill("confirmCount", String(at + 1), String(waiting.length))}
        </p>
      )}
      <div className="rounded-[10px] border border-hair bg-panel px-3.5 py-3">
        <span className="block text-[13px] font-semibold">{name}</span>
        <span className="block text-[11.5px] text-faint">
          {[one.os, since && fill("confirmSince", since)].filter(Boolean).join(" · ")}
        </span>
      </div>
      <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("confirmWhat")}</p>
      <p className="mt-2 text-[12.5px] leading-relaxed">{t("confirmStep")}</p>
      <div className="mt-3 rounded-[10px] border border-hair bg-panel px-3.5 py-3">
        <span className="block text-[11.5px] text-faint">{fill("confirmCodeOf", name)}</span>
        <span className="mt-1 block font-mono text-[21px] tracking-[0.06em] tabular-nums">
          {one.code ?? t("confirmNoCode")}
        </span>
      </div>
      {trouble && (
        <p role="alert" className="mt-2 text-[12.5px] text-urgent">
          {trouble}
        </p>
      )}
      <p className="mt-3 text-[13px] font-semibold">{t("confirmAsk")}</p>
      {why && <p className="mt-2 text-[12.5px] leading-relaxed text-soft">{t("confirmWhyIs")}</p>}
      <div className="mt-4 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
        <button
          type="button"
          onClick={() => setWhy((was) => !was)}
          className="mr-auto text-faint underline decoration-hair hover:text-ink"
        >
          {t("confirmWhyAsk")}
        </button>
        <button
          type="button"
          onClick={done}
          className="rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
        >
          {t("knockLater")}
        </button>
        <button
          type="button"
          onClick={() => setStep("mismatch")}
          className="rounded-[10px] border border-line px-3 py-1.5 hover:border-ink"
        >
          {t("confirmNo")}
        </button>
        <button
          type="button"
          disabled={!one.code}
          onClick={yes}
          className="rounded-[10px] bg-accent px-3.5 py-1.5 font-semibold text-bg disabled:opacity-60"
        >
          {t("confirmYes")}
        </button>
      </div>
      {here?.code && (
        <p className="mt-4 border-t border-hair pt-3 text-[11.5px] leading-relaxed text-faint">
          {fill("confirmHere", here.name ?? t("machineHere"), here.code)}
        </p>
      )}
    </Modal>
  );
}
