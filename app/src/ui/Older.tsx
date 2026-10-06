import { useState } from "react";
import { updateInstall, updateReady } from "../core";
import { fill, t } from "../locales";

interface Props {
  name?: string;
  busy: boolean;
  onStay: () => void;
  onOther: () => void;
  onError: (problem: unknown) => void;
}

export default function Older({ name, busy, onStay, onOther, onError }: Props) {
  const [looking, setLooking] = useState(false);
  const [found, setFound] = useState<string>();
  const waiting = busy || looking;

  const update = () => {
    setLooking(true);
    setFound(undefined);
    updateReady(true)
      .then((ready) => {
        if (ready?.installs) return updateInstall();
        setFound(
          t(!ready ? "lookNowNone" : ready.route === "store" ? "updateLanded" : "updateComingWhy"),
        );
      })
      .catch(onError)
      .finally(() => setLooking(false));
  };

  return (
    <div className="flex flex-col gap-3">
      <div
        role="alert"
        className="rounded-[10px] border border-hue-amber/40 px-3 py-2 text-[11.5px] leading-relaxed text-soft"
      >
        <span className="block text-[12.5px] font-semibold text-ink">{t("welcomeOlder")}</span>
        {name ? fill("welcomeOlderBy", name) : t("welcomeOlderWhy")}
      </div>
      {found && (
        <p role="status" className="text-[11.5px] text-soft">
          {found}
        </p>
      )}
      <div className="flex items-center justify-end gap-3">
        <button
          type="button"
          disabled={waiting}
          onClick={onOther}
          className="text-[12.5px] text-faint hover:text-ink disabled:opacity-60"
        >
          {t("welcomeOtherFolder")}
        </button>
        <button
          type="button"
          disabled={waiting}
          onClick={update}
          className="rounded-[10px] border border-hair px-4 py-2 text-[13px] text-ink hover:bg-hover disabled:opacity-60"
        >
          {t(looking ? "lookingNow" : "updateInstall")}
        </button>
        <button
          type="button"
          disabled={waiting}
          onClick={onStay}
          className="rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60"
        >
          {t("welcomeStayHere")}
        </button>
      </div>
    </div>
  );
}
