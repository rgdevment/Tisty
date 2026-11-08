import { useEffect, useState } from "react";
import { type Identity, identityBegin, identityShown } from "../identity";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import { PhraseTyped } from "./Identity";
import Modal from "./Modal";
import Phrasing from "./Phrasing";
import { mild, strong } from "./Rows";

export default function FirstIdentity({
  onReady,
  onClose,
}: Readonly<{ onReady: () => void; onClose: () => void }>) {
  const [shown, setShown] = useState<Identity | null>(null);
  const [words, setWords] = useState<string[] | null>(null);
  const [making, setMaking] = useState(false);
  const [trouble, setTrouble] = useState<string | null>(null);
  const failed = (problem: unknown) => setTrouble(saidPlainly(problem));

  useEffect(() => {
    identityShown()
      .then(setShown)
      .catch((problem) => setTrouble(saidPlainly(problem)));
  }, []);

  const make = () => {
    setMaking(true);
    setTrouble(null);
    identityBegin()
      .then((phrase) => {
        setWords(phrase.split(" "));
        return identityShown().then(setShown);
      })
      .catch(failed)
      .finally(() => setMaking(false));
  };

  if (words && shown?.fingerprint)
    return (
      <Phrasing
        words={words}
        fingerprint={shown.fingerprint}
        onClose={onReady}
        onConfirmed={onReady}
      />
    );

  return (
    <Modal title={t("shareIdentityTitle")} onClose={onClose} wide>
      <div className="flex flex-col gap-3">
        <p className="text-[12.5px] leading-relaxed text-soft">{t("shareIdentityWhy")}</p>
        {shown?.elsewhere && (
          <p className="text-[12.5px] leading-relaxed">{t("shareIdentityElsewhere")}</p>
        )}
        {trouble && (
          <p role="alert" className="text-[12.5px] text-urgent">
            {trouble}
          </p>
        )}
        <div className="flex flex-wrap items-start justify-end gap-2">
          <button type="button" className={`mr-auto ${mild}`} onClick={onClose}>
            {t("cancel")}
          </button>
          {shown?.elsewhere ? (
            <PhraseTyped held={making} onTyped={onReady} onFail={failed} />
          ) : (
            <button type="button" className={strong} disabled={making || !shown} onClick={make}>
              {t("shareIdentityMake")}
            </button>
          )}
        </div>
      </div>
    </Modal>
  );
}
