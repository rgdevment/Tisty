import { useState } from "react";
import {
  ALIAS_AT_MOST,
  guide,
  keepClosing,
  keepLocale,
  sign,
  sowLists,
  syncNow,
  wakeFor,
} from "../core";
import { adopt, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Keepers from "./Keepers";
import Modal from "./Modal";

interface Props {
  onDone: (paper?: string) => void;
}

type Step = "tongue" | "signing" | "copies";

const STEPS: Step[] = ["tongue", "signing", "copies"];

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
  const [tongue, setTongue] = useState<string>();
  const [alias, setAlias] = useState("");
  const [deciding, setDeciding] = useState(false);

  const at = STEPS.indexOf(step);

  const speak = (code: string) => {
    setBusy(true);
    setTrouble(undefined);
    keepLocale(code)
      .then(() => adopt(code))
      .then(() => {
        setTongue(code);
        setStep("signing");
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const signAs = () => {
    const said = alias.trim();
    setBusy(true);
    setTrouble(undefined);
    (said ? sign(said) : Promise.resolve(null))
      .then(() => setStep("copies"))
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const leave = (at?: string) => {
    setBusy(true);
    setTrouble(undefined);
    Promise.allSettled([wakeFor(true), keepClosing("hide")])
      .then(() => (at ? syncNow().catch(() => undefined) : undefined))
      .then(() => sowLists().catch(() => undefined))
      .then(() =>
        guide()
          .then((paper) => paper.id as string | undefined)
          .catch(() => undefined),
      )
      .then(onDone);
  };

  return (
    <Modal
      title={
        step === "tongue"
          ? t("welcomeTongue")
          : step === "signing"
            ? t("welcomeSigning")
            : t("welcomeCopies")
      }
      wide={step === "copies"}
    >
      <div
        role="progressbar"
        aria-label={fill("welcomeStep", `${at + 1}`)}
        aria-valuemin={1}
        aria-valuemax={STEPS.length}
        aria-valuenow={at + 1}
        className="mt-3 flex items-center gap-1.5"
      >
        {STEPS.map((one, n) => (
          <span
            key={one}
            className={`h-1.5 rounded-full ${
              n === at ? "w-4 bg-accent" : "w-1.5 bg-line"
            } motion-safe:transition-all`}
          />
        ))}
      </div>

      <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
        {step === "tongue"
          ? t("welcomeTongueWhy")
          : step === "signing"
            ? t("welcomeSigningWhy")
            : t("keepersWhy")}
      </p>

      <div className="mt-5 flex flex-col gap-2">
        {step === "signing" ? (
          <>
            <p className="text-[12.5px] leading-relaxed text-soft">{t("welcomeSigningHow")}</p>
            <div className="flex gap-2">
              <input
                type="text"
                aria-label={t("alias")}
                value={alias}
                disabled={busy}
                maxLength={ALIAS_AT_MOST}
                placeholder={t("aliasLike")}
                onChange={(e) => setAlias(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && signAs()}
                className="min-w-0 flex-1 rounded-lg border border-line bg-bg px-3 py-2 text-[13px] disabled:opacity-60"
              />
              <button
                type="button"
                disabled={busy}
                onClick={signAs}
                className="cursor-pointer rounded-lg bg-accent px-3.5 py-2 text-[13px] text-bg disabled:opacity-60"
              >
                {t("welcomeSigned")}
              </button>
            </div>
            <p className="text-[11.5px] leading-relaxed text-faint">{t("welcomeSigningNote")}</p>
          </>
        ) : step === "tongue" ? (
          TONGUES.map((one) => (
            <Choice
              key={one.code}
              said={one.name}
              busy={busy}
              on={tongue === one.code}
              onPick={() => speak(one.code)}
            />
          ))
        ) : (
          <Keepers busy={busy} onTrouble={setTrouble} onDeciding={setDeciding} onDone={leave} />
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
        {step === "copies" && !deciding && (
          <button
            type="button"
            disabled={busy}
            onClick={() => setStep("signing")}
            className="text-faint hover:text-ink disabled:opacity-60"
          >
            {t("welcomeBack")}
          </button>
        )}
        {step === "signing" && (
          <>
            <button
              type="button"
              disabled={busy}
              onClick={() => setStep("tongue")}
              className="text-faint hover:text-ink disabled:opacity-60"
            >
              {t("welcomeBack")}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => setStep("copies")}
              className="ml-auto text-faint hover:text-ink disabled:opacity-60"
            >
              {t("welcomeNotNow")}
            </button>
          </>
        )}
        {step === "tongue" && (
          <button
            type="button"
            disabled={busy}
            onClick={() => setStep("signing")}
            className="ml-auto text-faint hover:text-ink disabled:opacity-60"
          >
            {t("welcomeNext")}
          </button>
        )}
      </div>
    </Modal>
  );
}
