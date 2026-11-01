import { useEffect, useId, useRef, useState } from "react";
import { copied } from "../core";
import { fill, t } from "../locales";
import { type ShareCard, type ShareSecret, shareDoc, shareLook } from "../shares";
import Modal from "./Modal";

interface Props {
  file: string;
  title: string;
  onShared: (card: ShareCard) => void;
  onClose: () => void;
  onError: (problem: unknown) => void;
}

// A few are enough to send the person looking; the rest are only counted.
const SECRETS_SHOWN = 3;

// What is encrypted ends on its own after 30 idle days; the dialog only lets it end sooner (decisions 49 and 50).
const IDLE_DAYS = [30, 14, 7];

const choice =
  "flex cursor-pointer gap-2.5 rounded-[10px] border px-3.5 py-2.5 has-[:checked]:border-accent has-[:checked]:bg-accent-soft";

export default function Sharing({ file, title, onShared, onClose, onError }: Props) {
  const [gated, setGated] = useState(false);
  const [password, setPassword] = useState("");
  const [idle, setIdle] = useState(IDLE_DAYS[0]);
  const [busy, setBusy] = useState(false);
  const [secrets, setSecrets] = useState<ShareSecret[]>([]);
  // Undefined until the look answers, so a look that failed says nothing about the name rather than something false.
  const [signsAs, setSignsAs] = useState<string | null | undefined>(undefined);
  // Nothing goes out before the look is over, so the warning never comes after the press.
  const [looked, setLooked] = useState(false);
  const [seen, setSeen] = useState<string | null>(null);
  // Bumped when the words changed between the look and the press, so they are looked through again.
  const [round, setRound] = useState(0);
  const [changed, setChanged] = useState(false);
  // Read at the press itself, so a second Enter before the dialog redraws shares nothing twice.
  const going = useRef(false);
  const passwordId = useId();
  const idleId = useId();

  // A document that cannot be looked through still shares; the warning is only a second look.
  useEffect(() => {
    let live = true;
    setLooked(false);
    setSecrets([]);
    setSignsAs(undefined);
    setSeen(null);
    shareLook(file)
      .then((look) => {
        if (!live) return;
        setSecrets(look.secrets);
        setSignsAs(look.signsAs);
        setSeen(look.seen);
      })
      .catch(() => {})
      .finally(() => live && setLooked(true));
    return () => {
      live = false;
    };
  }, [file, round]);

  const ready = looked && !busy && (!gated || password.trim() !== "");

  const go = (event: React.FormEvent) => {
    event.preventDefault();
    if (!ready || going.current) return;
    going.current = true;
    setBusy(true);
    shareDoc(file, {
      password: gated ? password : null,
      // Only reading for now: editing a share from Tisty comes later, and an edit link must not promise it (decision 67).
      editable: false,
      idleDays: idle === IDLE_DAYS[0] ? null : idle,
      seen,
    })
      .then((card) => {
        copied(card.read).catch(() => {});
        onShared(card);
      })
      .catch((problem) => {
        going.current = false;
        setBusy(false);
        if ((problem as { code?: string } | null)?.code === "shareChanged") {
          setChanged(true);
          setRound((now) => now + 1);
          return;
        }
        onError(problem);
      });
  };

  return (
    <Modal title={t("shareTitle")} onClose={busy ? undefined : onClose} wide>
      <p className="-mt-2 mb-4 truncate text-[12.5px] text-soft">{title}</p>
      <form onSubmit={go} className="flex flex-col gap-5">
        {changed && (
          <p role="status" className="text-[12.5px] text-soft">
            {t("shareChanged")}
          </p>
        )}
        {secrets.length > 0 && (
          <div
            role="alert"
            className="flex flex-col gap-1 rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5"
          >
            <span className="text-[13px] font-semibold text-hue-amber">{t("shareSecrets")}</span>
            <ul className="text-[12.5px]">
              {secrets.slice(0, SECRETS_SHOWN).map((one) => (
                <li key={one.line}>
                  {one.under ? fill("shareSecretUnder", one.under) : t("shareSecretTop")}
                  {one.named && <code className="ml-1.5 font-mono text-[11.5px]">{one.named}</code>}
                </li>
              ))}
              {secrets.length > SECRETS_SHOWN && (
                <li>{fill("shareSecretsMore", String(secrets.length - SECRETS_SHOWN))}</li>
              )}
            </ul>
            <span className="text-[11.5px] leading-relaxed text-soft">{t("shareSecretsWhy")}</span>
          </div>
        )}
        <fieldset className="flex flex-col gap-2">
          <legend className="mb-2 text-[12.5px] font-semibold">{t("shareWho")}</legend>
          <label className={choice}>
            <input type="radio" name="who" checked={!gated} onChange={() => setGated(false)} />
            <span className="flex flex-col">
              <span className="text-[13px] font-medium">{t("shareUnlisted")}</span>
              <span className="text-[11.5px] text-faint">{t("shareUnlistedWhy")}</span>
            </span>
          </label>
          <label className={choice}>
            <input type="radio" name="who" checked={gated} onChange={() => setGated(true)} />
            <span className="flex flex-col">
              <span className="text-[13px] font-medium">{t("shareGated")}</span>
              <span className="text-[11.5px] text-faint">{t("shareGatedWhy")}</span>
            </span>
          </label>
          {gated && (
            <div className="flex flex-col gap-1 pl-7">
              <label htmlFor={passwordId} className="text-[11.5px] text-faint">
                {t("sharePassword")}
              </label>
              <input
                id={passwordId}
                type="password"
                autoComplete="new-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="h-8 rounded-md border border-line bg-bg px-2.5 text-[13px]"
              />
            </div>
          )}
        </fieldset>

        <p className="flex flex-col">
          <span className="text-[13px] font-medium">{t("shareSign")}</span>
          <span className="text-[11.5px] text-faint">
            {signsAs === undefined
              ? ""
              : signsAs
                ? fill("shareSignsAs", signsAs)
                : t("shareSignsAlone")}{" "}
            {t("shareSignWhy")}
          </span>
        </p>

        <div className="flex items-center justify-between gap-3">
          <label htmlFor={idleId} className="text-[13px] font-medium">
            {t("shareEnds")}
          </label>
          <select
            id={idleId}
            value={idle}
            onChange={(e) => setIdle(Number(e.target.value))}
            className="h-8 rounded-md border border-line bg-bg px-2 text-[12.5px]"
          >
            {IDLE_DAYS.map((days) => (
              <option key={days} value={days}>
                {fill("shareEndsIdle", String(days))}
              </option>
            ))}
          </select>
        </div>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            disabled={busy}
            className="h-8 rounded-md border border-line px-3 text-[12.5px] hover:bg-hover"
          >
            {t("cancel")}
          </button>
          <button
            type="submit"
            disabled={!ready}
            className="h-8 rounded-md bg-accent px-3 text-[12.5px] font-semibold text-white disabled:opacity-50"
          >
            {t(
              busy
                ? "shareGoing"
                : !looked
                  ? "shareLooking"
                  : secrets.length > 0
                    ? "shareGoAnyway"
                    : "shareGo",
            )}
          </button>
        </div>
      </form>
    </Modal>
  );
}
