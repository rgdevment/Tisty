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
import Card, { type Word } from "./Card";
import Phrasing from "./Phrasing";
import { mild, strong } from "./Rows";
import { held as isHeld, type Working } from "./working";

interface Props {
  round?: string | null;
  phrased?: number;
  onPhrased?: () => void;
  onTyped?: () => void;
  /** The cloud waits for its key below, where the same words open it without being kept. */
  opensCloud?: boolean;
  busy: Working;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  clear?: (card: "identity") => void;
  onShown?: (shown: Shown) => void;
}

export default function Identity({
  round,
  phrased,
  onPhrased,
  onTyped,
  opensCloud,
  busy,
  tell,
  fail,
  clear,
  onShown,
}: Props) {
  const held = isHeld(busy, "identity");
  const [shown, setShown] = useState<Shown | null>(null);
  const [words, setWords] = useState<string[] | null>(null);

  const [unread, setUnread] = useState(false);
  const look = useCallback(() => {
    setUnread(false);
    identityShown()
      .then((now) => {
        setShown(now);
        if (now) onShown?.(now);
      })
      .catch((e) => {
        setUnread(true);
        fail({
          card: "identity",
          text: saidPlainly(e),
          again: () => {
            clear?.("identity");
            look();
          },
        });
      });
  }, [fail, clear, onShown]);
  // A finished round may certify a computer, and the encryption card may confirm the phrase.
  useEffect(look, [look, round, phrased]);

  const see = () => {
    identityPhrase()
      .then((phrase) => setWords(phrase.split(" ")))
      .catch((e) => fail({ card: "identity", text: saidPlainly(e) }));
  };

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
    onTyped?.();
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
      <Card row="identity" which="identity">
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
          </>
        )}
        {shown && born && <Born shown={shown} held={held} onSee={see} />}
        {shown && !shown.held && (
          <div className="mt-3 border-t border-hair pt-3">
            <p className="text-[12.5px] leading-relaxed text-soft">{t("phraseTypeWhat")}</p>
            {opensCloud && (
              <p className="mt-1 text-[11.5px] leading-relaxed text-faint">
                {t("phraseTypeOther")}
              </p>
            )}
            <PhraseTyped held={held} onTyped={typed} onFail={failed} />
          </div>
        )}
      </Card>
    </>
  );
}

// The phrase typed on another computer of the person's, or again after its key was taken back, gives this one its key.
export function PhraseTyped({
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
  const onlyHere = useId();
  if (!open)
    return (
      <button
        type="button"
        disabled={held}
        onClick={() => setOpen(true)}
        className={`mt-3 ${mild}`}
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
      <p id={onlyHere} className="text-[11.5px] font-semibold">
        {t("phraseTypeOnlyHere")}
      </p>
      <textarea
        id={fieldId}
        aria-describedby={onlyHere}
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

function Born({ shown, held, onSee }: { shown: Shown; held: boolean; onSee: () => void }) {
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
      {shown.held && (
        <div className="mt-3 flex flex-wrap justify-end gap-2">
          <button type="button" className={strong} disabled={held} onClick={onSee}>
            {t("identitySee")}
          </button>
        </div>
      )}
    </>
  );
}
