import type { ParcelAnswer } from "../core";
import { filled, t } from "../locales";
import Digits, { HOW_MANY } from "./Digits";
import Modal from "./Modal";

export type Question =
  | { kind: "ifMine"; at: string; number?: string }
  | { kind: "signedBy"; at: string; number?: string; name: string; print: string };

interface Props {
  asks: {
    whoFor: string | null;
    movingTo: string | null;
    locked: string | null;
    number: string;
    wrong: boolean;
    question: Question | null;
  };
  onQuestion: (question: Question | null) => void;
  onAnswer: (answer: ParcelAnswer) => void;
  onWho: (named: string | null) => void;
  onMoving: (named: string | null) => void;
  onLocked: (at: string | null) => void;
  onNumber: (said: string) => void;
  onWrong: (wrong: boolean) => void;
  onShare: (named: string) => void;
  onLockAndPack: () => void;
  onOpenLocked: () => void;
}

export default function ParcelAsks({
  asks,
  onQuestion,
  onAnswer,
  onWho,
  onMoving,
  onLocked,
  onNumber,
  onWrong,
  onShare,
  onLockAndPack,
  onOpenLocked,
}: Props) {
  return (
    <>
      {asks.whoFor !== null && (
        <Modal title={t("packWho")} onClose={() => onWho(null)}>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("packWhoWhy")}</p>
          <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => onWho(null)}
              className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              onClick={() => {
                const named = asks.whoFor;
                onWho(null);
                if (named !== null) onShare(named);
              }}
              className="cursor-pointer rounded-[10px] border border-line px-3 py-1.5 text-ink hover:bg-line/40"
            >
              {t("packToShare")}
            </button>
            <button
              type="button"
              onClick={() => {
                onNumber("");
                onMoving(asks.whoFor);
                onWho(null);
              }}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg"
            >
              {t("packToMove")}
            </button>
          </div>
        </Modal>
      )}

      {asks.movingTo !== null && (
        <Modal
          title={t("packToMove")}
          onClose={() => {
            onMoving(null);
            onNumber("");
          }}
        >
          <p className="mt-3 text-[12.5px] text-soft">{t("packNumber")}</p>
          <Digits
            label={t("packNumber")}
            value={asks.number}
            onChange={onNumber}
            onDone={onLockAndPack}
          />
          <p className="mt-3 text-[11.5px] leading-relaxed text-faint">{t("packNumberWhy")}</p>
          <div className="mt-5 flex items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => {
                onMoving(null);
                onNumber("");
              }}
              className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              disabled={asks.number.length < HOW_MANY}
              onClick={onLockAndPack}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg disabled:opacity-60"
            >
              {t("packLockIt")}
            </button>
          </div>
        </Modal>
      )}

      {asks.locked !== null && (
        <Modal
          title={t("parcelShut")}
          onClose={() => {
            onLocked(null);
            onNumber("");
          }}
        >
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("parcelLocked")}</p>
          <p className="mt-4 text-[12.5px] text-soft">{t("openNumber")}</p>
          <Digits
            label={t("openNumber")}
            value={asks.number}
            onChange={(said) => {
              onWrong(false);
              onNumber(said);
            }}
            onDone={onOpenLocked}
          />
          {asks.wrong && (
            <p role="alert" className="mt-3 text-[11.5px] text-urgent">
              {t("wrongNumber")}
            </p>
          )}
          <div className="mt-5 flex items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => {
                onLocked(null);
                onNumber("");
              }}
              className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              disabled={asks.number.length < HOW_MANY}
              onClick={onOpenLocked}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg disabled:opacity-60"
            >
              {t("openLocked")}
            </button>
          </div>
        </Modal>
      )}

      {asks.question?.kind === "ifMine" && (
        <Modal title={t("parcelIfMineTitle")} onClose={() => onQuestion(null)}>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("parcelIfMine")}</p>
          <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => onQuestion(null)}
              className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              onClick={() => onAnswer("theirs")}
              className="cursor-pointer rounded-[10px] border border-line px-3 py-1.5 text-ink hover:bg-line/40"
            >
              {t("parcelGivenToMe")}
            </button>
            <button
              type="button"
              onClick={() => onAnswer("mine")}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg"
            >
              {t("parcelLockedByMe")}
            </button>
          </div>
        </Modal>
      )}

      {asks.question?.kind === "signedBy" && (
        <Modal title={t("parcelSignedByTitle")} onClose={() => onQuestion(null)}>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
            {filled(t("parcelSignedBy"), asks.question.name, asks.question.print)}
          </p>
          <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => onAnswer("theirs")}
              className="cursor-pointer rounded-[10px] border border-line px-3 py-1.5 text-ink hover:bg-line/40"
            >
              {t("parcelAsTheirs")}
            </button>
            <button
              type="button"
              onClick={() => onQuestion(null)}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg"
            >
              {t("parcelPhraseFirst")}
            </button>
          </div>
        </Modal>
      )}
    </>
  );
}
