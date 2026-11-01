import { fill, t } from "../../locales";
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
}

export default function Problem({
  trouble,
  name,
  place,
  onReveal,
  onBrowse,
  onRetry,
  onEdit,
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
