import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { fill, t, type Word } from "../locales";
import Modal from "./Modal";

type Question = { id: number } & (
  | { kind: "look" }
  | { kind: "keep"; title: string; owner: string; signer: string; print: string; doubtful: boolean }
  | { kind: "failed"; code: string }
);

// The second press of a double click must not answer the question that opens under it.
const READ_FOR_AT_LEAST_MS = 400;

const FAILED: Record<string, Word> = {
  folioGone: "linkFailedGone",
  folioBlocked: "linkFailedBlocked",
  folioPaused: "linkFailedPaused",
  folioOutdated: "linkFailedOutdated",
  folioUnopened: "linkFailedUnopened",
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

function Keeping({
  asked,
  onAnswer,
}: Readonly<{
  asked: Extract<Question, { kind: "keep" }>;
  onAnswer: (yes: boolean) => void;
}>) {
  return (
    <Modal title={t("linkKeepTitle")} onClose={() => onAnswer(false)}>
      <div className="flex flex-col gap-3 text-[13px] leading-relaxed">
        <p className="break-words font-semibold">{asked.title || t("untitledDoc")}</p>
        {asked.owner && asked.owner !== asked.signer && (
          <p className="break-words">{fill("linkKeepOwner", asked.owner)}</p>
        )}
        {asked.signer && <p className="break-words">{fill("linkKeepSigned", asked.signer)}</p>}
        <div className="flex flex-col gap-1">
          <span className="text-[11.5px] text-soft">{t("linkKeepPrint")}</span>
          <code className="break-all rounded-md bg-hover px-2 py-1.5 font-mono text-[11.5px]">
            {asked.print}
          </code>
          <span className="text-[11.5px] text-soft">{t("linkKeepCompare")}</span>
        </div>
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
  return (
    <Modal title={t("linkLookTitle")} onClose={() => answer(false)}>
      <div className="flex flex-col gap-4">
        <p className="text-[13px] leading-relaxed">{t("linkLookWhy")}</p>
        <Answers yes={t("linkLookGo")} no={t("linkNotNow")} onAnswer={answer} />
      </div>
    </Modal>
  );
}
