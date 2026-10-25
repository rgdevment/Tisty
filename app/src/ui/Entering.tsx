import { useEffect, useState } from "react";
import { type Heard, sealingHeard, sealingPhrase } from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Modal from "./Modal";
import { mild, strong } from "./Rows";

export default function Entering({ onClose, onIn }: { onClose: () => void; onIn: () => void }) {
  const [said, setSaid] = useState("");
  const [heard, setHeard] = useState<Heard[]>([]);
  const [trouble, setTrouble] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    sealingHeard()
      .then(setHeard)
      .catch(() => setHeard([]));
  }, []);

  const enter = () => {
    setBusy(true);
    setTrouble(null);
    sealingPhrase(said.trim())
      .then(() => {
        setSaid("");
        onIn();
        onClose();
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  return (
    <Modal title={t("enterTitle")} wide onClose={onClose}>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("enterWhat")}</p>
      {heard.length > 0 && (
        <>
          <p className="mt-3 text-[12.5px] leading-relaxed">{t("enterOthers")}</p>
          <ul className="mt-1.5 flex flex-col gap-1 text-[12.5px]">
            {heard.map((one) => (
              <li key={`${one.name}-${one.at}`} className="flex gap-2">
                <span className="min-w-0 flex-1 truncate">
                  {[one.name, one.os].filter(Boolean).join(" · ")}
                </span>
                <span className="shrink-0 text-faint">
                  {fill("enterSeen", stamped(new Date(one.at * 1000).toISOString()))}
                </span>
              </li>
            ))}
          </ul>
        </>
      )}
      <textarea
        aria-label={t("enterTitle")}
        value={said}
        onChange={(event) => setSaid(event.target.value)}
        rows={4}
        spellCheck={false}
        autoComplete="off"
        className="mt-3 w-full rounded-[10px] border border-line bg-bg px-2.5 py-2 font-mono text-[12.5px]"
      />
      {trouble && (
        <p role="alert" className="mt-2 text-[12.5px] text-urgent">
          {trouble}
        </p>
      )}
      <div className="mt-4 flex flex-wrap justify-end gap-2">
        <button type="button" className={mild} onClick={onClose}>
          {t("cancel")}
        </button>
        <button
          type="button"
          className={strong}
          disabled={busy || said.trim().length === 0}
          onClick={enter}
        >
          {t("enterGo")}
        </button>
      </div>
    </Modal>
  );
}
