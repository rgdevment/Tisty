import { useEffect, useState } from "react";
import { type Identity, identityBegin, identityShown } from "../identity";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import { PhraseTyped } from "./Identity";
import Modal from "./Modal";
import Phrasing from "./Phrasing";
import { mild, strong } from "./Rows";

const EXPLAINED = [
  ["shareIdentityWhatHead", "shareIdentityWhatSaid"],
  ["shareIdentityForHead", "shareIdentityForSaid"],
  ["shareIdentityFolioHead", "shareIdentityFolioSaid"],
  ["shareIdentityPhraseHead", "shareIdentityPhraseSaid"],
  ["shareIdentityLostHead", "shareIdentityLostSaid"],
] as const;

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
        <p className="text-[12.5px] leading-relaxed">{t("shareIdentityWhy")}</p>
        <details className="rounded-[10px] border border-hair px-3.5 py-2.5 text-[12.5px]">
          <summary className="cursor-pointer font-semibold">{t("shareIdentityMore")}</summary>
          <dl className="mt-2 flex flex-col gap-2 leading-relaxed">
            {EXPLAINED.map(([head, said]) => (
              <div key={head}>
                <dt className="font-semibold">{t(head)}</dt>
                <dd className="text-soft">{t(said)}</dd>
              </div>
            ))}
          </dl>
        </details>
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
