import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { chooseSync, wakeFor, waking } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";

interface Props {
  onDone: () => void;
}

export default function Welcome({ onDone }: Props) {
  const [busy, setBusy] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const [offered, setOffered] = useState(false);
  const [wakes, setWakes] = useState(true);
  const was = useRef(false);

  useEffect(() => {
    waking()
      .then((now) => {
        was.current = now.wakes;
        setOffered(now.offered);
      })
      .catch(() => {});
  }, []);

  const leave = () => {
    if (offered && wakes !== was.current) void wakeFor(wakes).catch(() => {});
    onDone();
  };

  const settle = (dest?: string) => {
    setBusy(true);
    chooseSync(dest)
      .then(leave)
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const pick = () => {
    if (busy) return;
    setBusy(true);
    setTrouble(undefined);
    open({ directory: true })
      .then((at) => {
        if (typeof at === "string") return settle(at);
        setBusy(false);
      })
      .catch((e) => {
        setTrouble(saidPlainly(e));
        setBusy(false);
      });
  };

  return (
    <Modal title={t("welcomeTitle")}>
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

      {offered && (
        <label className="mt-3.5 flex items-center gap-2 text-[12.5px]">
          <input type="checkbox" checked={wakes} onChange={(e) => setWakes(e.target.checked)} />
          {t("wakeAdd")}
        </label>
      )}

      {trouble && (
        <p role="alert" className="mt-3 text-xs text-urgent">
          {trouble}
        </p>
      )}

      <p className="mt-4 text-xs leading-relaxed text-faint">{t("welcomeRedundancy")}</p>

      <button type="button" onClick={leave} className="mt-4 text-xs text-faint hover:text-ink">
        {t("welcomeLater")}
      </button>
    </Modal>
  );
}
