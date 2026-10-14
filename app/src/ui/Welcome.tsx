import { useCallback, useEffect, useRef, useState } from "react";
import { stillApart, walkThrough } from "../apart";
import {
  ALIAS_AT_MOST,
  chooseSync,
  guide,
  joining,
  type Kin,
  keepClosing,
  keepLocale,
  placePeek,
  type Settled,
  sign,
  sowLists,
  syncKin,
  syncNow,
  syncState,
  wakeFor,
} from "../core";
import { adopt, fill, t } from "../locales";
import { folderAhead, saidPlainly } from "../refusal";
import { causeSaid } from "../syncSaid";
import Apart, { type Door } from "./Apart";
import Bringing from "./Bringing";
import Choosing from "./Choosing";
import Modal from "./Modal";
import Older from "./Older";

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

const NAME_COMING_TRIES = 10;
const NAME_COMING_EVERY = 3000;

export default function Welcome({ onDone }: Props) {
  const [step, setStep] = useState<Step>("tongue");
  const [busy, setBusy] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const [tongue, setTongue] = useState<string>();
  const [alias, setAlias] = useState("");
  const [deciding, setDeciding] = useState(false);
  const [carrying, setCarrying] = useState(false);
  const [stuck, setStuck] = useState<{ said: string; by?: unknown }>();
  const [kin, setKin] = useState<Kin>();
  const [named, setNamed] = useState<string>();
  const [offered, setOffered] = useState(false);
  const [another, setAnother] = useState(false);
  const [toCloud, setToCloud] = useState(false);
  const went = useRef(false);
  const joined = useRef(false);

  useEffect(() => {
    syncState()
      .then((state) => setOffered(state.cloudOffered))
      .catch(() => undefined);
  }, []);

  const ahead = folderAhead(stuck?.by);
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
    const settling = Promise.allSettled([wakeFor(true), keepClosing("hide")]);
    if (joined.current) return settling.then(() => onDone(undefined));
    return settling
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

  // A machine that held only what it starts with took the place's history, so nothing is sown on top of it.
  const round = (left = TRIES): Promise<Settled | undefined> =>
    syncNow().then((how) => {
      if (how?.adopted) joined.current = true;
      return how?.carried === "busy" && left > 0
        ? new Promise<void>((soon) => setTimeout(soon, BREATH)).then(() => round(left - 1))
        : how;
    });

  // Read once the round brought the place home, so asking for it never pulls the whole place first.
  const signedThere = (tries = 0): Promise<string | undefined> =>
    joining()
      .then((how) =>
        how.coming && tries < NAME_COMING_TRIES
          ? new Promise<void>((done) => window.setTimeout(done, NAME_COMING_EVERY)).then(() =>
              signedThere(tries + 1),
            )
          : (how.alias ?? undefined),
      )
      .catch(() => undefined);

  const carryOn = (name?: string) => {
    setBusy(true);
    setTrouble(undefined);
    setStuck(undefined);
    setCarrying(true);
    return round()
      .then(async (how) => {
        if (went.current) return;
        if (how?.carried === "busy") {
          setCarrying(false);
          setStuck({ said: t("syncBusy") });
          return;
        }
        const away = how?.deferred?.reason === "offline" ? causeSaid(how.deferred.cause) : null;
        if (away || how?.deferred?.reason === "troubled") {
          setCarrying(false);
          setStuck({ said: away ?? t("deferredTroubled") });
          return;
        }
        const signed = name ?? (joined.current ? await signedThere() : undefined);
        if (signed) setNamed(signed);
        return next(signed);
      })
      .catch((e) => {
        if (went.current) return;
        setCarrying(false);
        if (!stillApart(e)) return setStuck({ said: saidPlainly(e), by: e });
        void syncKin()
          .catch(() => "unsure" as const)
          .then(setKin);
      })
      .finally(() => setBusy(false));
  };

  // A machine that never wrote anything takes the place's Tisty without asking, and is told so while it comes.
  const settle = (): Promise<void> =>
    placePeek()
      .then((peeked) => {
        if (peeked.fresh && peeked.holds) joined.current = true;
      })
      .catch(() => undefined)
      .then(() => {
        setBusy(false);
        return carryOn();
      });

  const chose = (at?: string) => {
    setToCloud(at === "cloud");
    setTrouble(undefined);
    setStuck(undefined);
    if (!at) {
      setStep("signing");
      return;
    }
    setBusy(true);
    void settle();
  };

  const stayHere = () => {
    setBusy(true);
    chooseSync()
      .then(() => {
        joined.current = false;
        setNamed(undefined);
        chose();
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const closed = (door: Door | "else" | null) => {
    setKin(undefined);
    if (door === null) return setStuck({ said: t("wouldReset") });
    joined.current = door === "merge" || door === "theirs";
    setCarrying(true);
    walkThrough(door)
      .then((gone) => {
        if (!gone) return setStuck({ said: t("wouldReset") });
        if (door === "else") return settle();
        return round().then(() => next());
      })
      .catch((e) => {
        if (went.current) return;
        setStuck({ said: saidPlainly(e), by: e });
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
        cloud={toCloud ? t("cloudGoogle") : undefined}
        onPick={(door) => closed(door)}
        onElse={() => closed("else")}
        onClose={() => closed(null)}
      />
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
        ) : stuck && ahead ? (
          <Older
            name={ahead.name}
            busy={busy}
            onStay={stayHere}
            onOther={() => {
              setAnother(true);
              joined.current = false;
              setNamed(undefined);
              setStuck(undefined);
            }}
            onError={(e) => setTrouble(saidPlainly(e))}
          />
        ) : stuck ? (
          <div className="flex flex-col gap-3">
            <div
              role="alert"
              className="rounded-[10px] border border-hue-amber/40 px-3 py-2 text-[11.5px] leading-relaxed text-soft"
            >
              <span className="block text-[12.5px] font-semibold text-ink">
                {t("welcomeCarryStuck")}
              </span>
              {stuck.said}
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
          <Bringing
            from={toCloud ? t("cloudGoogle") : t("syncTheFolder")}
            onHide={() => void next(named)}
          />
        ) : (
          <Choosing
            busy={busy}
            offered={offered}
            start={another ? "folder" : "pick"}
            onTrouble={setTrouble}
            onDeciding={setDeciding}
            onDone={chose}
          />
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
