import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { chooseSync } from "../core";
import { t } from "../locales";

interface Props {
  onDone: () => void;
  onError: (problem: unknown) => void;
}

export default function Welcome({ onDone, onError }: Props) {
  const [busy, setBusy] = useState(false);

  const settle = (dest?: string) => {
    setBusy(true);
    chooseSync(dest)
      .then(onDone)
      .catch(onError)
      .finally(() => setBusy(false));
  };

  const pick = () => {
    open({ directory: true })
      .then((at) => typeof at === "string" && settle(at))
      .catch(onError);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink/25 p-6">
      <div className="w-full max-w-md rounded-xl border border-hair bg-bg p-6 shadow-xl">
        <h2 className="text-lg font-semibold">{t("welcomeTitle")}</h2>
        <p className="mt-2 text-[12.5px] leading-relaxed text-soft">{t("welcomeWhy")}</p>

        <div className="mt-5 flex flex-col gap-2">
          <button
            type="button"
            disabled={busy}
            onClick={() => settle(undefined)}
            className="rounded-lg border border-line px-3.5 py-2.5 text-left hover:bg-hover"
          >
            <span className="block text-[13.5px] font-medium">{t("welcomeAlone")}</span>
            <span className="block text-xs text-faint">{t("welcomeAloneWhy")}</span>
          </button>

          <button
            type="button"
            disabled={busy}
            onClick={pick}
            className="rounded-lg border border-line px-3.5 py-2.5 text-left hover:bg-hover"
          >
            <span className="block text-[13.5px] font-medium">{t("welcomeShared")}</span>
            <span className="block text-xs text-faint">{t("welcomeSharedWhy")}</span>
          </button>
        </div>

        <p className="mt-4 text-xs leading-relaxed text-faint">{t("welcomeRedundancy")}</p>

        <button
          type="button"
          onClick={onDone}
          className="mt-4 text-xs text-faint hover:text-ink"
        >
          {t("welcomeLater")}
        </button>
      </div>
    </div>
  );
}
