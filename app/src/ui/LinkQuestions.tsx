import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { fill, t, type Word } from "../locales";
import { tongueName } from "../tongues";
import Modal from "./Modal";

type Standing =
  | { standing: "verified"; maker: string; domain?: string | null }
  | { standing: "imitates" }
  | { standing: "takenBack" }
  | { standing: "checked"; domain: string }
  | { standing: "plain" };

type Party = { name: string; print: string | null } & Standing;

type Question = { id: number } & (
  | { kind: "look" }
  | {
      kind: "keep";
      title: string;
      owner: Party | null;
      signer: Party;
      doubtful: boolean;
    }
  | { kind: "failed"; code: string }
  | { kind: "languagesLeft"; languages: string[] }
);

// The second press of a double click must not answer the question that opens under it.
const READ_FOR_AT_LEAST_MS = 400;

const FAILED: Record<string, Word> = {
  folioGone: "linkFailedGone",
  folioBlocked: "linkFailedBlocked",
  folioPaused: "linkFailedPaused",
  folioOutdated: "linkFailedOutdated",
  folioUnopened: "linkFailedUnopened",
  folioEditLinkMismatch: "linkFailedEditLink",
};

const QUIET = "h-8 rounded-md border border-line px-3 text-[12.5px] hover:bg-hover";
const MAIN = "h-8 rounded-md bg-accent px-3 text-[12.5px] font-semibold text-white";

function Answers({
  yes,
  no,
  onAnswer,
}: Readonly<{ yes: string; no?: string; onAnswer: (yes: boolean) => void }>) {
  return (
    <div className="flex justify-end gap-2">
      {no && (
        <button type="button" onClick={() => onAnswer(false)} className={QUIET}>
          {no}
        </button>
      )}
      <button type="button" onClick={() => onAnswer(true)} className={MAIN}>
        {yes}
      </button>
    </div>
  );
}

// The maker is named by Folio's word, never by the alias the envelope carries.
function Who({ party, said }: Readonly<{ party: Party; said: Word }>) {
  const name = party.standing === "verified" ? party.maker : party.name;
  if (!name) return null;
  return (
    <div className="flex flex-col gap-0.5">
      <p className="break-words">
        {fill(said, name)}
        {party.standing === "verified" && (
          <span className="ml-2 rounded-full bg-hue-green/10 px-2 py-0.5 text-[11.5px] font-semibold text-hue-green">
            {t("linkKeepVerified")}
          </span>
        )}
      </p>
      {party.standing === "verified" && party.domain && (
        <span className="break-all text-[11.5px] text-soft">{party.domain}</span>
      )}
      {party.standing === "imitates" && (
        <span className="text-[11.5px] font-semibold text-hue-red">{t("linkKeepOtherKey")}</span>
      )}
      {party.standing === "takenBack" && (
        <span className="text-[11.5px] font-semibold text-hue-red">{t("linkKeepTakenBack")}</span>
      )}
      {party.standing === "checked" && (
        <span className="break-all text-[11.5px] text-soft">
          {fill("linkKeepChecked", party.domain)}
        </span>
      )}
    </div>
  );
}

function Print({ party, said }: Readonly<{ party: Party; said: Word }>) {
  if (!party.print) return null;
  return (
    <div className="flex flex-col gap-1">
      <span className="text-[11.5px] text-soft">{t(said)}</span>
      <code className="rounded-md bg-hover px-2 py-1.5 font-mono text-[12.5px] tracking-wider tabular-nums">
        {party.print}
      </code>
    </div>
  );
}

function Keeping({
  asked,
  onAnswer,
}: Readonly<{
  asked: Extract<Question, { kind: "keep" }>;
  onAnswer: (yes: boolean) => void;
}>) {
  const prints = [asked.owner, asked.signer].filter((one) => one?.print).length;
  return (
    <Modal title={t("linkKeepTitle")} onClose={() => onAnswer(false)}>
      <div className="flex flex-col gap-3 text-[13px] leading-relaxed">
        <p className="break-words font-semibold">{asked.title || t("untitledDoc")}</p>
        {asked.owner && (
          <section className="flex flex-col gap-1.5">
            <Who party={asked.owner} said="linkKeepOwner" />
            <Print party={asked.owner} said="linkKeepOwnerPrint" />
          </section>
        )}
        <section className="flex flex-col gap-1.5">
          <Who party={asked.signer} said="linkKeepSigned" />
          <Print party={asked.signer} said="linkKeepPrint" />
        </section>
        {prints > 0 && (
          <span className="text-[11.5px] text-soft">
            {t(prints > 1 ? "linkKeepCompareEach" : "linkKeepCompare")}
          </span>
        )}
        {asked.doubtful && (
          <p
            role="alert"
            className="rounded-[10px] bg-hue-amber/10 px-3.5 py-2.5 text-[12.5px] text-hue-amber"
          >
            {t("linkKeepDoubt")}
          </p>
        )}
        <Answers yes={t("linkKeepGo")} no={t("linkNotNow")} onAnswer={onAnswer} />
      </div>
    </Modal>
  );
}

export default function LinkQuestions() {
  const [asked, setAsked] = useState<Question | null>(null);
  const [looking, setLooking] = useState(false);
  const shownAt = useRef(0);

  const look = useCallback(() => {
    invoke<Question | null>("link_question_standing")
      .then((now) => {
        if (!now) return;
        shownAt.current = Date.now();
        setLooking(false);
        setAsked(now);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    // Read only once it listens: a question asked in between would never be heard of.
    const off = listen("linkquestion", look);
    void off.then(look);
    return () => {
      void off.then((stop) => stop());
    };
  }, [look]);

  const answer = (yes: boolean) => {
    if (!asked || Date.now() - shownAt.current < READ_FOR_AT_LEAST_MS) return;
    setAsked(null);
    setLooking(yes && asked.kind === "look");
    void invoke("link_question_answered", { id: asked.id, yes }).catch(() => {});
  };

  if (looking)
    return (
      <Modal title={t("linkLookingTitle")} onClose={() => setLooking(false)}>
        <output className="text-[13px] leading-relaxed">{t("linkLookingWhy")}</output>
      </Modal>
    );
  if (!asked) return null;
  if (asked.kind === "keep") return <Keeping asked={asked} onAnswer={answer} />;
  if (asked.kind === "failed")
    return (
      <Modal title={t("linkFailedTitle")} onClose={() => answer(true)}>
        <div className="flex flex-col gap-4">
          <p className="text-[13px] leading-relaxed">
            {t(FAILED[asked.code] ?? "linkFailedUnreached")}
          </p>
          <Answers yes={t("linkFailedOk")} onAnswer={answer} />
        </div>
      </Modal>
    );
  if (asked.kind === "languagesLeft")
    return (
      <Modal title={t("linkLanguagesLeftTitle")} onClose={() => answer(true)}>
        <div className="flex flex-col gap-4">
          <p className="text-[13px] leading-relaxed">
            {fill("linkLanguagesLeft", asked.languages.map(tongueName).join(", "))}
          </p>
          <Answers yes={t("linkFailedOk")} onAnswer={answer} />
        </div>
      </Modal>
    );
  return (
    <Modal title={t("linkLookTitle")} onClose={() => answer(false)}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] leading-relaxed">{t("linkLookWhy")}</p>
        <Answers yes={t("linkLookGo")} no={t("linkNotNow")} onAnswer={answer} />
      </div>
    </Modal>
  );
}
