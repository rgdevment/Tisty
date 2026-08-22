import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { chooseSync, guide, keepClosing, keepLocale, wakeFor, waking } from "../core";
import { adopt, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";

interface Props {
  onDone: (paper?: string) => void;
}

type Step = "tongue" | "copies" | "wake" | "closing" | "ready";

const TONGUES = [
  { code: "es", name: "Español" },
  { code: "en", name: "English" },
];

function Choice({
  said,
  why,
  onPick,
  busy,
  on,
}: {
  said: string;
  why?: string;
  onPick: () => void;
  busy: boolean;
  on?: boolean;
}) {
  return (
    <button
      type="button"
      disabled={busy}
      aria-pressed={on}
      onClick={onPick}
      className={`rounded-lg border px-3.5 py-2.5 text-left hover:bg-hover disabled:opacity-60 ${
        on ? "border-accent bg-accent-soft" : "border-line"
      }`}
    >
      <span className="block text-[13.5px] font-medium">{said}</span>
      {why && <span className="block text-xs text-faint">{why}</span>}
    </button>
  );
}

export default function Welcome({ onDone }: Props) {
  const [step, setStep] = useState<Step>("tongue");
  const [busy, setBusy] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const [offered, setOffered] = useState(false);
  const [chose, setChose] = useState<Partial<Record<Step, string>>>({});
  const asked = useRef(false);

  useEffect(() => {
    waking()
      .then((now) => setOffered(now.offered))
      .catch(() => {});
  }, []);

  const steps: Step[] = ["tongue", "copies", "wake", "closing", "ready"].filter(
    (one) => one !== "wake" || offered,
  ) as Step[];
  const at = steps.indexOf(step);

  const ahead = () => {
    setTrouble(undefined);
    setStep(steps[at + 1] ?? "ready");
  };

  const back = () => {
    setTrouble(undefined);
    setStep(steps[Math.max(at - 1, 0)]);
  };

  const settle = (picked: string, work: Promise<unknown>) => {
    setBusy(true);
    setTrouble(undefined);
    work
      .then(() => {
        setChose((was) => ({ ...was, [step]: picked }));
        ahead();
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const speak = (code: string) => {
    settle(
      code,
      keepLocale(code).then(() => adopt(code)),
    );
  };

  const cloud = () => {
    if (busy || asked.current) return;
    asked.current = true;
    setBusy(true);
    setTrouble(undefined);
    open({ directory: true })
      .then((where) =>
        typeof where === "string"
          ? chooseSync(where).then(() => {
              setChose((was) => ({ ...was, copies: "cloud" }));
              ahead();
            })
          : undefined,
      )
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => {
        asked.current = false;
        setBusy(false);
      });
  };

  const leave = () => {
    setBusy(true);
    const settled = chose.copies ? Promise.resolve() : chooseSync(undefined).catch(() => undefined);
    settled
      .then(() => guide())
      .then((paper) => onDone(paper.id))
      .catch((e) => {
        setTrouble(saidPlainly(e));
        setBusy(false);
      });
  };

  const title = {
    tongue: t("welcomeTongue"),
    copies: t("welcomeCopies"),
    wake: t("wake"),
    closing: t("welcomeClosing"),
    ready: t("welcomeReady"),
  }[step];

  return (
    <Modal title={title}>
      <div
        role="progressbar"
        aria-label={fill("welcomeStep", `${at + 1}`)}
        aria-valuemin={1}
        aria-valuemax={steps.length}
        aria-valuenow={at + 1}
        className="mt-3 flex items-center gap-1.5"
      >
        {steps.map((one, n) => (
          <span
            key={one}
            className={`h-1.5 rounded-full ${
              n === at ? "w-4 bg-accent" : "w-1.5 bg-line"
            } motion-safe:transition-all`}
          />
        ))}
      </div>

      <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
        {step === "tongue" && t("welcomeTongueWhy")}
        {step === "copies" && t("welcomeWhy")}
        {step === "wake" && t("wakeOn")}
        {step === "closing" && t("closingWhy")}
        {step === "ready" && t("welcomeGuideWhy")}
      </p>

      <div className="mt-5 flex flex-col gap-2">
        {step === "tongue" &&
          TONGUES.map((one) => (
            <Choice
              key={one.code}
              said={one.name}
              busy={busy}
              on={chose.tongue === one.code}
              onPick={() => speak(one.code)}
            />
          ))}

        {step === "copies" && (
          <>
            <Choice
              said={t("welcomeCloud")}
              why={t("welcomeCloudWhy")}
              busy={busy}
              on={chose.copies === "cloud"}
              onPick={cloud}
            />
            <Choice
              said={t("welcomeAlone")}
              why={t("welcomeAloneWhy")}
              busy={busy}
              on={chose.copies === "alone"}
              onPick={() => settle("alone", chooseSync(undefined))}
            />
          </>
        )}

        {step === "wake" && (
          <>
            <Choice
              said={t("wakeAdd")}
              why={t("wakeFresh")}
              busy={busy}
              on={chose.wake === "yes"}
              onPick={() => settle("yes", wakeFor(true))}
            />
            <Choice
              said={t("wakeOff")}
              why={t("wakeGone")}
              busy={busy}
              on={chose.wake === "no"}
              onPick={() => settle("no", wakeFor(false))}
            />
          </>
        )}

        {step === "closing" && (
          <>
            <Choice
              said={t("closingHide")}
              why={t("closingHideWhy")}
              busy={busy}
              on={chose.closing === "hide"}
              onPick={() => settle("hide", keepClosing("hide"))}
            />
            <Choice
              said={t("closingQuit")}
              why={t("closingQuitWhy")}
              busy={busy}
              on={chose.closing === "quit"}
              onPick={() => settle("quit", keepClosing("quit"))}
            />
          </>
        )}

        {step === "ready" && (
          <button
            type="button"
            disabled={busy}
            onClick={leave}
            className="self-end rounded-lg bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60"
          >
            {t("welcomeGuide")}
          </button>
        )}
      </div>

      {trouble && (
        <p role="alert" className="mt-3 text-xs text-urgent">
          {trouble}
        </p>
      )}

      {step === "copies" && (
        <p className="mt-4 text-xs leading-relaxed text-faint">{t("welcomeRedundancy")}</p>
      )}

      <div className="mt-4 flex items-center gap-4 text-xs">
        {at > 0 && (
          <button type="button" onClick={back} className="text-faint hover:text-ink">
            {t("welcomeBack")}
          </button>
        )}
        {step !== "ready" && (
          <button type="button" onClick={ahead} className="ml-auto text-faint hover:text-ink">
            {step === "copies" && !chose.copies ? t("welcomeLater") : t("welcomeNext")}
          </button>
        )}
      </div>
    </Modal>
  );
}
