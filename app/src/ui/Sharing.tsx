import { useId, useState } from "react";
import { copied } from "../core";
import { fill, t } from "../locales";
import { type ShareCard, shareDoc } from "../shares";
import Modal from "./Modal";

interface Props {
  file: string;
  title: string;
  onShared: (card: ShareCard) => void;
  onClose: () => void;
  onError: (problem: unknown) => void;
}

// What is encrypted ends on its own after 30 idle days; the dialog only lets it end sooner (decisions 49 and 50).
const IDLE_DAYS = [30, 14, 7];

const choice =
  "flex cursor-pointer gap-2.5 rounded-[10px] border px-3.5 py-2.5 has-[:checked]:border-accent has-[:checked]:bg-accent-soft";

export default function Sharing({ file, title, onShared, onClose, onError }: Props) {
  const [gated, setGated] = useState(false);
  const [password, setPassword] = useState("");
  const [editable, setEditable] = useState(false);
  const [idle, setIdle] = useState(IDLE_DAYS[0]);
  const [busy, setBusy] = useState(false);
  const passwordId = useId();
  const idleId = useId();

  const ready = !busy && (!gated || password.trim() !== "");

  const go = (event: React.FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    setBusy(true);
    shareDoc(file, {
      password: gated ? password : null,
      editable,
      idleDays: idle === IDLE_DAYS[0] ? null : idle,
    })
      .then((card) => {
        copied(card.read).catch(() => {});
        onShared(card);
      })
      .catch((problem) => {
        setBusy(false);
        onError(problem);
      });
  };

  return (
    <Modal title={t("shareTitle")} onClose={busy ? undefined : onClose} wide>
      <p className="-mt-2 mb-4 truncate text-[12.5px] text-soft">{title}</p>
      <form onSubmit={go} className="flex flex-col gap-5">
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

        <fieldset className="flex flex-col gap-2">
          <legend className="mb-2 text-[12.5px] font-semibold">{t("shareWhat")}</legend>
          <div className="grid grid-cols-2 gap-1 rounded-[10px] bg-hover p-1">
            {[false, true].map((one) => (
              <label
                key={String(one)}
                className="flex h-8 cursor-pointer items-center justify-center gap-2 rounded-md text-[12.5px] has-[:checked]:bg-bg has-[:checked]:font-semibold has-[:checked]:shadow-lift"
              >
                <input
                  type="radio"
                  name="what"
                  className="sr-only"
                  checked={editable === one}
                  onChange={() => setEditable(one)}
                />
                {t(one ? "shareCanEdit" : "shareReadOnly")}
              </label>
            ))}
          </div>
          <p className="text-[11.5px] leading-relaxed text-faint">{t("shareEditWhy")}</p>
        </fieldset>

        <p className="flex flex-col">
          <span className="text-[13px] font-medium">{t("shareSign")}</span>
          <span className="text-[11.5px] text-faint">{t("shareSignWhy")}</span>
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
            {t(busy ? "shareGoing" : "shareGo")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
