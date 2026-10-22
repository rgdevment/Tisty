import { useState } from "react";
import { backUp, cloudForget, cloudPause, cloudRevoke, leaveHere, revealed } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import { backupMadeSaid, whereToBackUp } from "./Backup";
import Modal from "./Modal";
import { mild } from "./Rows";

interface Props {
  cloud: boolean;
  // Whether the cloud can take Tisty's access away on every computer at once.
  revokes: boolean;
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

export default function Leave({ cloud, revokes, store, onClose }: Readonly<Props>) {
  const [backing, setBacking] = useState<"idle" | "busy" | "done">("idle");
  const [short, setShort] = useState<string>();
  const [revoke, setRevoke] = useState(false);
  const [going, setGoing] = useState(false);
  const [trouble, setTrouble] = useState<string>();
  const [stayed, setStayed] = useState<string[]>([]);
  const [letGo, setLetGo] = useState(false);
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

  const failed = (e: unknown, gone: boolean) => {
    setTrouble(gone ? `${t("leaveCloudGone")} ${saidPlainly(e)}` : saidPlainly(e));
    setGoing(false);
  };

  // Paused, then forgotten before the sweep: forgetting is what clears the keychain, and no round starts in between.
  const leave = async () => {
    setGoing(true);
    setTrouble(undefined);
    let gone = letGo;
    try {
      if (cloud && !gone) {
        await cloudPause(true);
        await (revoke ? cloudRevoke() : cloudForget()).catch(async (e) => {
          await cloudPause(false).catch(() => undefined);
          throw e;
        });
        gone = true;
        setLetGo(true);
      }
      if (store) await revealed(store).catch(() => undefined);
      setStayed((await leaveHere()) ?? []);
      setGoing(false);
    } catch (e) {
      failed(e, gone);
    }
  };

  const anyway = () => {
    setGoing(true);
    setTrouble(undefined);
    leaveHere(true).catch((e) => failed(e, letGo));
  };

  if (stayed.length > 0) {
    return (
      <Modal title={t("leaveTitle")} wide>
        <p role="alert" className="text-[12.5px] leading-relaxed text-soft">
          {t("leaveHeldOpen")}
        </p>
        <ul className="mt-2 font-mono text-[11.5px] break-all text-ink">
          {stayed.map((at) => (
            <li key={at}>{at}</li>
          ))}
        </ul>
        {trouble && <p className="mt-2.5 text-[11.5px] text-urgent">{trouble}</p>}
        <div className="mt-4 flex justify-end">
          <button type="button" disabled={going} onClick={anyway} className={leaving}>
            {t("leaveAnyway")}
          </button>
        </div>
      </Modal>
    );
  }

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
            {revokes && (
              <label className="mt-1.5 flex items-center gap-2 text-ink">
                <input
                  type="checkbox"
                  checked={revoke}
                  disabled={held}
                  onChange={(e) => setRevoke(e.target.checked)}
                />
                {t("leaveRevoke")}
              </label>
            )}
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
