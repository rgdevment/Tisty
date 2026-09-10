import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { stillApart, walkThrough } from "../apart";
import {
  ALIAS_AT_MOST,
  guide,
  type Joining,
  joining,
  type Kin,
  keepClosing,
  keepLocale,
  sign,
  sowLists,
  syncKin,
  syncNow,
  wakeFor,
} from "../core";
import { adopt, fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Apart, { type Door } from "./Apart";
import Keepers from "./Keepers";
import Modal from "./Modal";

interface Props {
  onDone: (paper?: string) => void;
}

type Step = "tongue" | "copies" | "signing";

const STEPS: Step[] = ["tongue", "copies", "signing"];

const TONGUES = [
  { code: "es", name: "Español" },
  { code: "en", name: "English" },
];

const TRIES = 3;

const BREATH = 500;

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
      className={`rounded-[10px] border px-3.5 py-2.5 text-left hover:bg-hover disabled:opacity-60 ${
        on ? "border-accent bg-accent-soft" : "border-line"
      }`}
    >
      <span className="block text-[13px] font-medium">{said}</span>
      {why && <span className="block text-[11.5px] text-faint">{why}</span>}
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
  const [carrying, setCarrying] = useState(false);
  const [stuck, setStuck] = useState<string>();
  const [kin, setKin] = useState<Kin>();
  const [offer, setOffer] = useState<Joining>();
  const [named, setNamed] = useState<string>();
  const went = useRef(false);

  const path = named ? STEPS.filter((one) => one !== "signing") : STEPS;
  const at = Math.min(path.indexOf(step), path.length - 1);

  const speak = (code: string) => {
    setBusy(true);
    setTrouble(undefined);
    keepLocale(code)
      .then(() => adopt(code))
      .then(() => {
        setTongue(code);
        setStep("copies");
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const finish = useCallback(() => {
    if (went.current) return Promise.resolve();
    went.current = true;
    return Promise.allSettled([wakeFor(true), keepClosing("hide")])
      .then(() => sowLists())
      .catch(() => undefined)
      .then(() =>
        guide()
          .then((paper) => paper.id as string | undefined)
          .catch(() => undefined),
      )
      .then(onDone);
  }, [onDone]);

  const next = useCallback(
    (name?: string) => {
      if (name ?? named) return finish();
      setCarrying(false);
      setStep("signing");
      return Promise.resolve();
    },
    [finish, named],
  );

  useEffect(() => {
    if (!carrying) return;
    const off = listen("carried", () => {
      void next();
    });
    return () => {
      off.then((stop) => stop()).catch(() => {});
    };
  }, [carrying, next]);

  const round = (left = TRIES): Promise<string | undefined> =>
    syncNow().then((how) =>
      how?.carried === "busy" && left > 0
        ? new Promise((soon) => setTimeout(soon, BREATH)).then(() => round(left - 1))
        : how?.carried,
    );

  const carryOn = (name?: string) => {
    setBusy(true);
    setTrouble(undefined);
    setStuck(undefined);
    setCarrying(true);
    return round()
      .then((how) => {
        if (how !== "busy") return next(name);
        setCarrying(false);
        setStuck(t("syncBusy"));
        return Promise.resolve();
      })
      .catch((e) => {
        if (went.current) return;
        setCarrying(false);
        if (!stillApart(e)) return setStuck(saidPlainly(e));
        syncKin()
          .catch(() => "unsure" as const)
          .then(setKin);
      })
      .finally(() => setBusy(false));
  };

  const settle = () =>
    joining()
      .then((how) => {
        setBusy(false);
        if (!how.fresh || !how.holds) return carryOn();
        setCarrying(false);
        setOffer(how);
      })
      .catch(() => {
        setBusy(false);
        return carryOn();
      });

  const chose = (at?: string) => {
    setTrouble(undefined);
    setStuck(undefined);
    if (!at) {
      setStep("signing");
      return;
    }
    setBusy(true);
    void settle();
  };

  const takeItAll = () => {
    const name = offer?.alias ?? undefined;
    setOffer(undefined);
    setNamed(name);
    void carryOn(name);
  };

  const closed = (door: Door | "else" | null) => {
    setKin(undefined);
    if (door === null) return setStuck(t("wouldReset"));
    setCarrying(true);
    walkThrough(door)
      .then((gone) => {
        if (!gone) return setStuck(t("wouldReset"));
        if (door === "else") return settle();
        return round().then(() => next());
      })
      .catch((e) => {
        if (!went.current) setStuck(saidPlainly(e));
      })
      .finally(() => {
        if (!went.current) setCarrying(false);
      });
  };

  const signAs = () => {
    const said = alias.trim();
    setBusy(true);
    setTrouble(undefined);
    (said ? sign(said) : Promise.resolve(null))
      .then(() => finish())
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  if (kin) {
    return (
      <Apart
        kin={kin}
        onPick={(door) => closed(door)}
        onElse={() => closed("else")}
        onClose={() => closed(null)}
      />
    );
  }

  if (offer) {
    return (
      <Modal title={t("welcomeFolderHolds")}>
        <p className="mt-3 text-[13px] leading-relaxed text-soft">{t("welcomeFolderHoldsWhy")}</p>
        <p className="mt-2 text-[12.5px] leading-relaxed text-faint">
          {offer.alias ? fill("welcomeFolderHoldsAs", offer.alias) : t("welcomeFolderHoldsHow")}
        </p>
        <div className="mt-5 flex items-center gap-3 text-[11.5px]">
          <button
            type="button"
            onClick={() => {
              setOffer(undefined);
              syncKin()
                .catch(() => "unsure" as const)
                .then(setKin);
            }}
            className="text-faint hover:text-ink"
          >
            {t("welcomeMoreDoors")}
          </button>
          <button
            type="button"
            onClick={takeItAll}
            className="ml-auto cursor-pointer rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-bg"
          >
            {t("welcomeBringIt")}
          </button>
        </div>
      </Modal>
    );
  }

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
        aria-valuemax={path.length}
        aria-valuenow={at + 1}
        className="mt-3 flex items-center gap-1.5"
      >
        {path.map((one, n) => (
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
                className="min-w-0 flex-1 rounded-[10px] border border-line bg-bg px-3 py-2 text-[13px] disabled:opacity-60"
              />
              <button
                type="button"
                disabled={busy}
                onClick={signAs}
                className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-2 text-[13px] text-bg disabled:opacity-60"
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
        ) : stuck ? (
          <div className="flex flex-col gap-3">
            <div
              role="alert"
              className="rounded-[10px] border border-hue-amber/40 px-3 py-2 text-[11.5px] leading-relaxed text-soft"
            >
              <span className="block text-[12.5px] font-semibold text-ink">
                {t("welcomeCarryStuck")}
              </span>
              {stuck}
            </div>
            <button
              type="button"
              onClick={() => next()}
              className="ml-auto rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90"
            >
              {t("welcomeAnyway")}
            </button>
          </div>
        ) : carrying ? (
          <div
            role="status"
            aria-live="polite"
            className="rounded-[10px] border border-hair bg-accent-soft px-3 py-2 text-[11.5px] leading-relaxed text-soft"
          >
            <span className="block text-[12.5px] font-semibold text-ink">
              {t("welcomeCarrying")}
            </span>
            {t("welcomeCarryingWhy")}
            <span
              aria-hidden="true"
              className="mt-2 block h-1 overflow-hidden rounded-full bg-line"
            >
              <span className="sliding block h-full w-1/3 rounded-full bg-accent" />
            </span>
          </div>
        ) : (
          <Keepers busy={busy} onTrouble={setTrouble} onDeciding={setDeciding} onDone={chose} />
        )}
      </div>

      {trouble && (
        <p role="alert" className="mt-3 text-[11.5px] text-urgent">
          {trouble}
        </p>
      )}

      {step === "copies" && (
        <p className="mt-4 text-[11.5px] leading-relaxed text-faint">{t("welcomeRedundancy")}</p>
      )}

      <div className="mt-4 flex items-center gap-4 text-[11.5px]">
        {step === "copies" && !deciding && !carrying && (
          <button
            type="button"
            disabled={busy}
            onClick={() => setStep("tongue")}
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
              onClick={() => setStep("copies")}
              className="text-faint hover:text-ink disabled:opacity-60"
            >
              {t("welcomeBack")}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={() => finish()}
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
            onClick={() => setStep("copies")}
            className="ml-auto text-faint hover:text-ink disabled:opacity-60"
          >
            {t("welcomeNext")}
          </button>
        )}
      </div>
    </Modal>
  );
}
