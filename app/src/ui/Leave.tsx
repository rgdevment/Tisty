import { useState } from "react";
import { backUp, cloudForget, cloudRevoke, leaveHere, revealed } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import { backupMadeSaid, whereToBackUp } from "./Backup";
import Modal from "./Modal";
import { mild } from "./Rows";

interface Props {
  cloud: boolean;
  store?: string;
  onClose: () => void;
}

const leaving =
  "rounded-[10px] bg-urgent px-2.5 py-1 text-[12.5px] text-bg disabled:border-hair disabled:bg-hair disabled:text-soft";

function Step({ n, children }: { n: number; children: React.ReactNode }) {
  return (
    <li className="flex gap-2.5 border-b border-hair py-2.5">
      <span className="w-4 shrink-0 text-[12.5px] text-faint tabular-nums">{n}.</span>
      <div className="min-w-0 flex-1 text-[12.5px] leading-relaxed text-soft">{children}</div>
    </li>
  );
}

export default function Leave({ cloud, store, onClose }: Props) {
  const [backing, setBacking] = useState<"idle" | "busy" | "done">("idle");
  const [short, setShort] = useState<string>();
  const [revoke, setRevoke] = useState(false);
  const [going, setGoing] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const held = going || backing === "busy";

  const backup = () => {
    setTrouble(undefined);
    whereToBackUp()
      .then(async (at) => {
        if (typeof at !== "string") return;
        setBacking("busy");
        const made = await backUp(at);
        setShort(made.leftOut ? backupMadeSaid(made) : undefined);
        setBacking("done");
      })
      .catch((e) => {
        setBacking("idle");
        setTrouble(saidPlainly(e));
      });
  };

  // The cloud lets go before the sweep, since forgetting the access is what clears it from the keychain.
  const leave = async () => {
    setGoing(true);
    setTrouble(undefined);
    let letGo = false;
    try {
      if (cloud) await (revoke ? cloudRevoke() : cloudForget());
      letGo = cloud;
      if (store) await revealed(store).catch(() => undefined);
      await leaveHere();
    } catch (e) {
      setTrouble(letGo ? `${t("leaveCloudGone")} ${saidPlainly(e)}` : saidPlainly(e));
      setGoing(false);
    }
  };

  const after = cloud ? 3 : 2;
  return (
    <Modal title={t("leaveTitle")} wide onClose={held ? undefined : onClose}>
      <ol className="border-t border-hair">
        <Step n={1}>
          <span className="flex items-center gap-2.5">
            <span className="flex-1">{t("leaveBackup")}</span>
            {backing === "done" ? (
              !short && (
                <span role="status" className="text-[12.5px] text-hue-green">
                  ✓ {t("leaveBackupDone")}
                </span>
              )
            ) : (
              <button type="button" disabled={held} onClick={backup} className={mild}>
                {t("backupMake")}
              </button>
            )}
          </span>
          {short && (
            <span role="status" className="mt-1 block text-hue-amber">
              {short}
            </span>
          )}
        </Step>
        {cloud && (
          <Step n={2}>
            <span className="block">{t("leaveCloud")}</span>
            <label className="mt-1.5 flex items-center gap-2 text-ink">
              <input
                type="checkbox"
                checked={revoke}
                disabled={held}
                onChange={(e) => setRevoke(e.target.checked)}
              />
              {t("leaveRevoke")}
            </label>
          </Step>
        )}
        <Step n={after}>{t("leaveLeft")}</Step>
        <Step n={after + 1}>{t("leaveThen")}</Step>
      </ol>
      {trouble && (
        <p role="alert" className="mt-2.5 text-[11.5px] text-urgent">
          {trouble}
        </p>
      )}
      <div className="mt-4 flex justify-end gap-2">
        <button type="button" disabled={held} onClick={onClose} className={mild}>
          {t("cancel")}
        </button>
        <button type="button" disabled={held} onClick={() => void leave()} className={leaving}>
          {t("leaveTitle")}
        </button>
      </div>
    </Modal>
  );
}
