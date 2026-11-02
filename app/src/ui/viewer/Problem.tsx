import { useId, useState } from "react";
import { fill, t } from "../../locales";
import { plainly } from "../../refusal";
import { Broken, button, tinted } from "./Glyphs";

export type Trouble =
  | { kind: "big"; limit: string }
  | { kind: "unreadable" }
  | { kind: "empty" }
  | { kind: "other"; said: string };

interface Props {
  trouble: Trouble;
  name: string;
  place: string;
  onReveal?: () => void;
  /** For a share Tisty could not show, the link opened where its reader can ask for a password. */
  onBrowse?: () => void;
  onRetry: () => void;
  onEdit?: () => void;
  /** For a share behind a password, Tisty asks for it here; a wrong one is said under the field. */
  onPassword?: (password: string) => Promise<unknown>;
}

function Unlocking({ onPassword }: { onPassword: (password: string) => Promise<unknown> }) {
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [wrong, setWrong] = useState<string | null>(null);
  const field = useId();
  const said = useId();
  const open = (event: React.FormEvent) => {
    event.preventDefault();
    if (busy || password === "") return;
    setBusy(true);
    setWrong(null);
    onPassword(password)
      .catch((problem) => setWrong(plainly(problem as { code: string })))
      .finally(() => setBusy(false));
  };
  return (
    <form onSubmit={open} className="mb-[18px] flex flex-col gap-1.5 text-left">
      <label htmlFor={field} className="text-[11.5px] text-faint">
        {t("sharePassword")}
      </label>
      <div className="flex gap-2">
        <input
          id={field}
          type="password"
          autoComplete="off"
          autoFocus
          value={password}
          aria-invalid={wrong !== null}
          aria-describedby={wrong ? said : undefined}
          onChange={(e) => setPassword(e.target.value)}
          className="h-8 min-w-0 flex-1 rounded-md border border-line bg-bg px-2.5 text-[13px]"
        />
        <button type="submit" disabled={busy || password === ""} className={tinted}>
          {t(busy ? "viewUnlocking" : "viewUnlock")}
        </button>
      </div>
      {wrong && (
        <p id={said} role="alert" className="text-[12.5px] text-hue-red">
          {wrong}
        </p>
      )}
    </form>
  );
}

export default function Problem({
  trouble,
  name,
  place,
  onReveal,
  onBrowse,
  onRetry,
  onEdit,
  onPassword,
}: Props) {
  const empty = trouble.kind === "empty";
  const title = empty
    ? t("viewEmptyTitle")
    : fill(trouble.kind === "unreadable" ? "viewUnreadTitle" : "viewBigTitle", name);
  const why =
    trouble.kind === "big"
      ? fill("viewBigWhy", trouble.limit)
      : trouble.kind === "unreadable"
        ? fill("viewUnreadWhy", place)
        : trouble.kind === "empty"
          ? t("viewEmptyWhy")
          : trouble.said;
  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <div className="max-w-[380px] text-center">
        <Broken empty={empty} />
        <h3 className="mb-1.5 text-[13px] font-semibold">{title}</h3>
        <p className="mb-[18px] text-[13px] leading-relaxed text-soft">{why}</p>
        {onPassword && <Unlocking onPassword={onPassword} />}
        <div className="flex justify-center gap-2">
          {empty ? (
            onEdit && (
              <button type="button" className={tinted} onClick={onEdit}>
                {t("viewEditIn")}
              </button>
            )
          ) : (
            <>
              {onReveal && (
                <button type="button" className={button} onClick={onReveal}>
                  {t("viewReveal")}
                </button>
              )}
              {onBrowse && (
                <button type="button" className={button} onClick={onBrowse}>
                  {t("viewBrowse")}
                </button>
              )}
              <button type="button" className={tinted} onClick={onRetry}>
                {t("viewRetry")}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
