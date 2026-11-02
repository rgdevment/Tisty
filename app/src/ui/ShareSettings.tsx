import { useId, useState } from "react";
import { fill, t } from "../locales";
import { type ShareCard, type ShareChange, shareChange } from "../shares";
import Modal from "./Modal";
import { IDLE_DAYS } from "./Sharing";

interface Props {
  file: string;
  card: ShareCard;
  onChanged: (card: ShareCard) => void;
  onClose: () => void;
  onError: (problem: unknown) => void;
}

const choice =
  "flex cursor-pointer gap-2.5 rounded-[10px] border px-3.5 py-2.5 has-[:checked]:border-accent has-[:checked]:bg-accent-soft";

// The server forgets the password it was given, so a share that keeps one only takes a new one when it is typed.
const changesOf = (
  card: ShareCard,
  gated: boolean,
  password: string,
  idle: number | null,
): ShareChange[] => {
  const changes: ShareChange[] = [];
  if (!gated && card.gated) changes.push({ do: "password", with: null });
  if (gated && password.trim() !== "") changes.push({ do: "password", with: password });
  if (idle !== null) changes.push({ do: "idle", days: idle });
  return changes;
};

export default function ShareSettings({ file, card, onChanged, onClose, onError }: Props) {
  const [gated, setGated] = useState(card.gated);
  const [password, setPassword] = useState("");
  const [idle, setIdle] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const passwordId = useId();
  const idleId = useId();
  const changes = changesOf(card, gated, password, idle);
  // Turning the password on asks for one; keeping it on may leave the field empty.
  const ready = !busy && changes.length > 0 && (!gated || card.gated || password.trim() !== "");

  const save = (event: React.FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    setBusy(true);
    changes
      .reduce<Promise<ShareCard | null>>(
        (before, one) => before.then(() => shareChange(file, one)),
        Promise.resolve(null),
      )
      .then((now) => {
        if (now) onChanged(now);
      })
      .catch((problem) => {
        setBusy(false);
        onError(problem);
      });
  };

  return (
    <Modal title={t("shareSettingsTitle")} onClose={busy ? undefined : onClose} wide>
      <form onSubmit={save} className="flex flex-col gap-5">
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
                {card.gated ? t("sharePasswordNew") : t("sharePassword")}
              </label>
              <input
                id={passwordId}
                type="password"
                autoComplete="new-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="h-8 rounded-md border border-line bg-bg px-2.5 text-[13px]"
              />
              {card.gated && (
                <span className="text-[11.5px] text-faint">{t("sharePasswordKept")}</span>
              )}
            </div>
          )}
        </fieldset>

        <div className="flex items-center justify-between gap-3">
          <label htmlFor={idleId} className="text-[13px] font-medium">
            {t("shareEnds")}
          </label>
          <select
            id={idleId}
            value={idle ?? ""}
            onChange={(e) => setIdle(e.target.value === "" ? null : Number(e.target.value))}
            className="h-8 rounded-md border border-line bg-bg px-2 text-[12.5px]"
          >
            <option value="">{t("shareEndsAsIs")}</option>
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
            {t(busy ? "shareSaving" : "saveIt")}
          </button>
        </div>
      </form>
    </Modal>
  );
}
