import { useCallback, useEffect, useId, useState } from "react";
import {
  identityBegin,
  identityPhrase,
  identityRecover,
  identityShown,
  type Identity as Shown,
} from "../identity";
import { t } from "../locales";
import { saidPlainly } from "../refusal";
import Card, { type Which, type Word } from "./Card";
import Phrasing from "./Phrasing";
import { mild, strong } from "./Rows";

interface Props {
  round?: string | null;
  phrased?: number;
  onPhrased?: () => void;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onSync: () => void;
}

export default function Identity({
  round,
  phrased,
  onPhrased,
  busy,
  said,
  trouble,
  tell,
  fail,
  onSync,
}: Props) {
  const held = busy !== null;
  const [shown, setShown] = useState<Shown | null>(null);
  const [words, setWords] = useState<string[] | null>(null);

  const [unread, setUnread] = useState(false);
  const look = useCallback(() => {
    setUnread(false);
    identityShown()
      .then(setShown)
      .catch((e) => {
        setUnread(true);
        fail({ card: "identity", text: saidPlainly(e) });
      });
  }, [fail]);
  // A finished round may certify a computer, and the encryption card may confirm the phrase.
  useEffect(look, [look, round, phrased]);

  const see = () => {
    identityPhrase()
      .then((phrase) => setWords(phrase.split(" ")))
      .catch((e) => fail({ card: "identity", text: saidPlainly(e) }));
  };

  // Sharing needs a key even with no cloud, so the phrase can be born here and not only while encrypting it.
  const [beginning, setBeginning] = useState(false);
  const begin = () => {
    setBeginning(true);
    identityBegin()
      .then((phrase) => {
        setWords(phrase.split(" "));
        // The phrase is shown beside its fingerprint, which only the identity born with it has.
        look();
      })
      .catch((e) => fail({ card: "identity", text: saidPlainly(e) }))
      .finally(() => setBeginning(false));
  };

  const failed = (e: unknown) => fail({ card: "identity", text: saidPlainly(e) });
  const typed = () => {
    look();
    onPhrased?.();
    tell({ card: "identity", text: t("phraseTyped") });
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
            onPhrased?.();
            tell({ card: "identity", text: t("phraseConfirmed") });
          }}
        />
      )}
      <Card title={t("tabIdentity")} which="identity" busy={busy} said={said} trouble={trouble}>
        {!shown && unread && (
          <button
            type="button"
            onClick={() => {
              if (trouble?.card === "identity") fail(undefined);
              look();
            }}
            className={mild}
          >
            {t("tryAgain")}
          </button>
        )}
        {!shown && !unread && (
          <p role="status" className="text-[12.5px] text-faint motion-safe:animate-pulse">
            {t("identityLooking")}
          </p>
        )}
        {shown && !born && (
          <>
            <p className="text-[12.5px] leading-relaxed text-soft">{t("identityUnborn")}</p>
            <button
              type="button"
              disabled={held || beginning}
              onClick={begin}
              className={`mt-3 ${strong}`}
            >
              {t("identityCreate")}
            </button>
            <PhraseTyped held={held} onTyped={typed} onFail={failed} />
          </>
        )}
        {shown && born && <Born shown={shown} held={held} onSee={see} onSync={onSync} />}
        {shown?.elsewhere && <PhraseTyped held={held} onTyped={typed} onFail={failed} />}
      </Card>
    </>
  );
}

// The phrase typed on another computer of the person's, or again after its key was taken back, gives this one its key.
function PhraseTyped({
  held,
  onTyped,
  onFail,
}: {
  held: boolean;
  onTyped: () => void;
  onFail: (problem: unknown) => void;
}) {
  const [open, setOpen] = useState(false);
  const [said, setSaid] = useState("");
  const [going, setGoing] = useState(false);
  const fieldId = useId();
  if (!open)
    return (
      <button
        type="button"
        disabled={held}
        onClick={() => setOpen(true)}
        className={`mt-3 ml-2 ${mild}`}
      >
        {t("phraseTypeOpen")}
      </button>
    );
  const send = (event: React.FormEvent) => {
    event.preventDefault();
    if (going || !said.trim()) return;
    setGoing(true);
    identityRecover(said.trim())
      .then(() => {
        setSaid("");
        setOpen(false);
        onTyped();
      })
      .catch(onFail)
      .finally(() => setGoing(false));
  };
  return (
    <form onSubmit={send} className="mt-3 flex flex-col gap-2">
      <label htmlFor={fieldId} className="text-[11.5px] text-faint">
        {t("phraseTypeHow")}
      </label>
      <textarea
        id={fieldId}
        value={said}
        onChange={(e) => setSaid(e.target.value)}
        rows={3}
        autoComplete="off"
        autoCorrect="off"
        spellCheck={false}
        className="rounded-[10px] border border-line bg-bg px-2.5 py-2 font-mono text-[12.5px]"
      />
      <div className="flex justify-end gap-2">
        <button type="button" onClick={() => setOpen(false)} className={mild}>
          {t("cancel")}
        </button>
        <button type="submit" disabled={held || going || !said.trim()} className={strong}>
          {t("phraseTypeGo")}
        </button>
      </div>
    </form>
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
