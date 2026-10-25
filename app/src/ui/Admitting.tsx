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

const LOOKED_EVERY = 3000;

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
    const look = () =>
      admittingShown()
        .then((said) => live && setNow(said))
        .catch((e) => live && setTrouble(saidPlainly(e)));
    admittingAsk()
      .then(look)
      .catch((e) => setTrouble(saidPlainly(e)));
    const every = setInterval(look, LOOKED_EVERY);
    return () => {
      live = false;
      clearInterval(every);
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
                {t("confirmNo")}
              </button>
              <button
                type="button"
                className={strong}
                disabled={!now.code}
                onClick={() =>
                  now.code && admittingTrust(now.code).catch((e) => setTrouble(saidPlainly(e)))
                }
              >
                {t("confirmYes")}
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

type Letting = { one: Newcomer; code: string; step: "compare" | "granted" | "refused" };

export function LetThemIn({ onClose }: { onClose: () => void }) {
  const [newcomers, setNewcomers] = useState<Newcomer[] | null>(null);
  const [letting, setLetting] = useState<Letting | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const failed = (e: unknown) => setTrouble(saidPlainly(e));

  const look = useCallback(() => {
    setTrouble(null);
    admittingLook()
      .then(setNewcomers)
      .catch((e) => setTrouble(saidPlainly(e)));
  }, []);
  useEffect(look, [look]);

  const take = (one: Newcomer) => {
    setTrouble(null);
    admittingOffer(one.device)
      .then((code) => setLetting({ one, code, step: "compare" }))
      .catch(failed);
  };

  const body = () => {
    if (letting?.step === "granted")
      return <p className="text-[12.5px] text-soft">{fill("admitGranted", letting.one.name)}</p>;
    if (letting?.step === "refused")
      return <p className="text-[12.5px] leading-relaxed">{t("admitMismatch")}</p>;
    if (letting)
      return (
        <>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {fill("admitCompare", letting.one.name)}
          </p>
          <Code code={letting.code} />
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
              {t("confirmNo")}
            </button>
            <button
              type="button"
              className={strong}
              onClick={() =>
                admittingGrant(letting.one.device, letting.code)
                  .then(() => setLetting({ ...letting, step: "granted" }))
                  .catch(failed)
              }
            >
              {t("confirmYes")}
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
                {t("admitShowCode")}
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
