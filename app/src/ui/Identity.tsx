import { useCallback, useEffect, useState } from "react";
import { identityPhrase, identityShown, type Identity as Shown } from "../core";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Card, { type Which, type Word } from "./Card";
import Phrasing from "./Phrasing";
import { mild, strong } from "./Rows";

interface Props {
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onSync: () => void;
}

export default function Identity({ busy, said, trouble, tell, fail, onSync }: Props) {
  const held = busy !== null;
  const [shown, setShown] = useState<Shown | null>(null);
  const [words, setWords] = useState<string[] | null>(null);

  const look = useCallback(() => {
    identityShown()
      .then(setShown)
      .catch((e) => fail({ card: "identity", text: saidPlainly(e) }));
  }, [fail]);
  useEffect(look, [look]);

  const see = () => {
    identityPhrase()
      .then((phrase) => setWords(phrase.split(" ")))
      .catch((e) => fail({ card: "identity", text: saidPlainly(e) }));
  };

  const born = shown !== null && (shown.held || shown.elsewhere);

  return (
    <>
      {words && shown?.fingerprint && (
        <Phrasing
          words={words}
          fingerprint={shown.fingerprint}
          onClose={() => setWords(null)}
          onConfirmed={() => {
            setWords(null);
            look();
            tell({ card: "identity", text: t("phraseConfirmed") });
          }}
        />
      )}
      <Card title={t("tabIdentity")} which="identity" busy={busy} said={said} trouble={trouble}>
        {shown && !born && (
          <p className="text-[12.5px] leading-relaxed text-soft">{t("identityUnborn")}</p>
        )}
        {shown && born && <Born shown={shown} held={held} onSee={see} onSync={onSync} />}
      </Card>
    </>
  );
}

function Born({
  shown,
  held,
  onSee,
  onSync,
}: {
  shown: Shown;
  held: boolean;
  onSee: () => void;
  onSync: () => void;
}) {
  const machines = shown.machines;
  const called = [shown.alias, shown.suffix].filter(Boolean).join(" · ");
  return (
    <>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("identityWhat")}</p>
      {shown.fingerprint && (
        <div className="mt-3 rounded-[10px] border border-hair px-3 py-2.5">
          <p className="font-mono text-[21px] tracking-wider">{shown.fingerprint}</p>
          <p className="mt-0.5 text-[11.5px] text-faint">{t("identityPrint")}</p>
        </div>
      )}
      <dl className="mt-3 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-[12.5px]">
        {called && (
          <>
            <dt className="text-faint">{t("identityName")}</dt>
            <dd>{called}</dd>
          </>
        )}
        {shown.held && (
          <>
            <dt className="text-faint">{t("identityPhrase")}</dt>
            <dd>
              <span
                className={`rounded-full border px-1.5 py-px text-[10.5px] font-semibold tracking-wide uppercase ${
                  shown.confirmed
                    ? "border-hue-teal text-hue-teal"
                    : "border-hue-amber text-hue-amber"
                }`}
              >
                {shown.confirmed ? t("identityConfirmed") : t("identityUnconfirmed")}
              </span>
            </dd>
          </>
        )}
      </dl>
      {shown.elsewhere && (
        <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("identityAway")}</p>
      )}
      {machines.length > 0 && (
        <>
          <p className="mt-3 text-[11.5px] font-semibold text-faint">{t("theMachines")}</p>
          <ul className="mt-1 flex flex-col gap-1 text-[12.5px]">
            {machines.map((one) => (
              <li
                key={one.id}
                className={`flex items-center justify-between gap-3 rounded-[10px] px-2.5 py-2 ${
                  one.here ? "bg-accent-soft" : ""
                }`}
              >
                <span className="min-w-0">
                  <span className="block text-[13px] font-semibold">{one.name}</span>
                  {one.here && <span className="block text-soft">{t("identityHere")}</span>}
                </span>
                {one.ours && (
                  <span className="rounded-full border border-accent px-1.5 py-px text-[10.5px] font-semibold tracking-wide text-accent uppercase">
                    {t("identityOurs")}
                  </span>
                )}
              </li>
            ))}
          </ul>
        </>
      )}
      <div className="mt-3 flex flex-wrap justify-end gap-2">
        <button type="button" className={mild} disabled={held} onClick={onSync}>
          {t("identityDrop")}
        </button>
        {shown.held && (
          <button type="button" className={strong} disabled={held} onClick={onSee}>
            {t("identitySee")}
          </button>
        )}
      </div>
    </>
  );
}
