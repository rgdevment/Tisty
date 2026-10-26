import { useCallback, useEffect, useRef, useState } from "react";
import {
  type Admitted,
  admittingAsk,
  admittingCancel,
  admittingGrant,
  admittingLook,
  admittingOffer,
  admittingRefuse,
  admittingShown,
  admittingTrust,
  type Newcomer,
} from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";
import { mild, strong } from "./Rows";

// Brisk while the person stands by both computers, then sparing, since every look is charged to the cloud's day.
const LOOKED_FIRST = 3000;
const LOOKED_LATER = 15000;
const BRISK_FOR = 60000;
const DIGITS = 20;
const GONE = ["admitGone", "admitTaken"];

function Code({ code }: { code: string }) {
  return (
    <p className="mt-3 rounded-[10px] border border-hair bg-panel px-3.5 py-3 text-center font-mono text-[21px] tracking-[0.06em] tabular-nums">
      {code}
    </p>
  );
}

function Trouble({ text }: { text: string | null }) {
  if (!text) return null;
  return (
    <p role="alert" className="mt-2 text-[12.5px] text-urgent">
      {text}
    </p>
  );
}

export function LetMeIn({ onClose, onIn }: { onClose: () => void; onIn: () => void }) {
  const [now, setNow] = useState<Admitted | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const [mismatch, setMismatch] = useState(false);

  useEffect(() => {
    let live = true;
    let next: ReturnType<typeof setTimeout> | undefined;
    const began = Date.now();
    const look = () =>
      admittingShown()
        .then((said) => live && setNow(said))
        .catch((e) => live && setTrouble(saidPlainly(e)))
        .finally(() => {
          if (!live) return;
          next = setTimeout(look, Date.now() - began < BRISK_FOR ? LOOKED_FIRST : LOOKED_LATER);
        });
    admittingAsk()
      .then(look)
      .catch((e) => setTrouble(saidPlainly(e)));
    return () => {
      live = false;
      clearTimeout(next);
    };
  }, []);

  // Told once: the card around it looks again on being told, and would tell it again.
  const told = useRef(false);
  useEffect(() => {
    if (now?.step !== "done" || told.current) return;
    told.current = true;
    onIn();
  }, [now?.step, onIn]);

  const close = () => {
    if (now?.step !== "done") void admittingCancel().catch(() => undefined);
    onClose();
  };

  const body = () => {
    if (mismatch) return <p className="text-[12.5px] leading-relaxed">{t("admitMismatch")}</p>;
    switch (now?.step) {
      case "compare":
        return (
          <>
            <p className="text-[12.5px] leading-relaxed text-soft">
              {fill("admitCompare", now.by ?? "")}
            </p>
            {now.code && <Code code={now.code} />}
            <div className="mt-4 flex flex-wrap justify-end gap-2">
              <button type="button" className={mild} onClick={() => setMismatch(true)}>
                {t("admitRejected")}
              </button>
              <button
                type="button"
                className={strong}
                disabled={!now.code}
                onClick={() =>
                  now.code && admittingTrust(now.code).catch((e) => setTrouble(saidPlainly(e)))
                }
              >
                {t("admitAccepted")}
              </button>
            </div>
          </>
        );
      case "trusted":
        return <p className="text-[12.5px] text-soft">{fill("admitTrusted", now.by ?? "")}</p>;
      case "done":
        return <p className="text-[12.5px] text-soft">{t("admitIn")}</p>;
      default:
        return <p className="text-[12.5px] leading-relaxed text-soft">{t("admitWaiting")}</p>;
    }
  };

  return (
    <Modal title={t("admitAsk")} wide onClose={close}>
      {body()}
      <Trouble text={trouble} />
      <div className="mt-4 flex justify-end">
        <button type="button" className={mild} onClick={close}>
          {t(now?.step === "done" ? "close" : "cancel")}
        </button>
      </div>
    </Modal>
  );
}

type Letting = { one: Newcomer; step: "typing" | "granted" | "refused" };

export function LetThemIn({ onClose }: { onClose: () => void }) {
  const [newcomers, setNewcomers] = useState<Newcomer[] | null>(null);
  const [letting, setLetting] = useState<Letting | null>(null);
  const [typed, setTyped] = useState("");
  const [granting, setGranting] = useState(false);
  const [trouble, setTrouble] = useState<string | null>(null);

  const look = useCallback(() => {
    setTrouble(null);
    admittingLook()
      .then(setNewcomers)
      .catch((e) => setTrouble(saidPlainly(e)));
  }, []);
  useEffect(look, [look]);

  // An ask gone or taken by another is shown as it now stands, rather than left on a form that can no longer work.
  const failed = (e: unknown) => {
    setTrouble(saidPlainly(e));
    const code = (e as { code?: string } | undefined)?.code;
    if (code && GONE.includes(code)) {
      setLetting(null);
      admittingLook()
        .then(setNewcomers)
        .catch(() => undefined);
    }
  };

  const take = (one: Newcomer) => {
    setTrouble(null);
    setTyped("");
    admittingOffer(one.device)
      .then(() => setLetting({ one, step: "typing" }))
      .catch(failed);
  };

  const count = typed.replace(/\D/g, "").length;

  const body = () => {
    if (letting?.step === "granted")
      return <p className="text-[12.5px] text-soft">{fill("admitGranted", letting.one.name)}</p>;
    if (letting?.step === "refused")
      return <p className="text-[12.5px] leading-relaxed">{t("admitMismatch")}</p>;
    if (letting)
      return (
        <>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {fill("admitTypeWhat", letting.one.name)}
          </p>
          <input
            aria-label={t("admitTypeLabel")}
            value={typed}
            inputMode="numeric"
            autoComplete="off"
            spellCheck={false}
            onChange={(event) => setTyped(event.target.value)}
            className="mt-3 w-full rounded-[10px] border border-line bg-bg px-3 py-2 text-center font-mono text-[18px] tracking-[0.06em] tabular-nums"
          />
          <div className="mt-4 flex flex-wrap justify-end gap-2">
            <button
              type="button"
              className={mild}
              onClick={() =>
                admittingRefuse(letting.one.device)
                  .then(() => setLetting({ ...letting, step: "refused" }))
                  .catch(failed)
              }
            >
              {t("admitNoComputer")}
            </button>
            <button
              type="button"
              className={strong}
              disabled={granting || count !== DIGITS}
              onClick={() => {
                setGranting(true);
                admittingGrant(letting.one.device, typed)
                  .then(() => setLetting({ ...letting, step: "granted" }))
                  .catch(failed)
                  .finally(() => setGranting(false));
              }}
            >
              {t("admitTypeGo")}
            </button>
          </div>
        </>
      );
    if (newcomers === null) return <p className="text-[12.5px] text-faint">{t("admitLooking")}</p>;
    if (newcomers.length === 0)
      return (
        <>
          <p className="text-[12.5px] leading-relaxed text-soft">{t("admitNobody")}</p>
          <div className="mt-3 flex justify-end">
            <button type="button" className={mild} onClick={look}>
              {t("admitLookAgain")}
            </button>
          </div>
        </>
      );
    return (
      <ul className="flex flex-col gap-2 text-[12.5px]">
        {newcomers.map((one) => (
          <li key={one.device} className="flex items-center gap-2">
            <span className="min-w-0 flex-1 truncate font-semibold">{one.name}</span>
            {one.taken ? (
              <span className="text-faint">{t("admitTaken")}</span>
            ) : (
              <button type="button" className={strong} onClick={() => take(one)}>
                {t("admitTake")}
              </button>
            )}
          </li>
        ))}
      </ul>
    );
  };

  return (
    <Modal title={t("admitLetIn")} wide onClose={onClose}>
      {body()}
      <Trouble text={trouble} />
      <div className="mt-4 flex justify-end">
        <button type="button" className={mild} onClick={onClose}>
          {t("close")}
        </button>
      </div>
    </Modal>
  );
}
