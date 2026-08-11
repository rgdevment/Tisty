import { useState } from "react";
import { closeWindow } from "../core";
import { t } from "../locales";

interface Props {
  onDismiss: () => void;
  onError: (problem: unknown) => void;
}

/** Asked the first time it matters, not buried in a settings screen. */
export default function Closing({ onDismiss, onError }: Props) {
  const [remember, setRemember] = useState(true);

  const settle = (how: "hide" | "quit") => {
    // Hiding leaves this mounted: it would return from the tray still open.
    closeWindow(how, remember).then(onDismiss).catch(onError);
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="closing-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-ink/25 p-6"
      onKeyDown={(e) => e.key === "Escape" && onDismiss()}
    >
      <div className="w-full max-w-sm rounded-xl border border-hair bg-bg p-6 shadow-xl">
        <h2 id="closing-title" className="text-lg font-semibold">
          {t("closingTitle")}
        </h2>
        <p className="mt-2 text-[12.5px] leading-relaxed text-soft">{t("closingWhy")}</p>

        <div className="mt-5 flex flex-col gap-2">
          <button
            type="button"
            autoFocus
            onClick={() => settle("hide")}
            className="rounded-lg border border-line px-3.5 py-2.5 text-left hover:bg-hover"
          >
            <span className="block text-[13.5px] font-medium">{t("closingHide")}</span>
            <span className="block text-xs text-faint">{t("closingHideWhy")}</span>
          </button>

          <button
            type="button"
            onClick={() => settle("quit")}
            className="rounded-lg border border-line px-3.5 py-2.5 text-left hover:bg-hover"
          >
            <span className="block text-[13.5px] font-medium">{t("closingQuit")}</span>
            <span className="block text-xs text-faint">{t("closingQuitWhy")}</span>
          </button>
        </div>

        <label className="mt-4 flex items-center gap-2 text-xs text-faint">
          <input
            type="checkbox"
            checked={remember}
            onChange={(e) => setRemember(e.target.checked)}
          />
          {t("closingRemember")}
        </label>

        <button type="button" onClick={onDismiss} className="mt-4 text-xs text-faint hover:text-ink">
          {t("closingStay")}
        </button>
      </div>
    </div>
  );
}
