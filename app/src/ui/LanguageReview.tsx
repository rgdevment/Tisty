import { convertFileSrc } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { fill, t } from "../locales";
import { type LanguageShown, type LanguageWord, languageReviewed, languageShown } from "../sharing";
import { tongueName } from "../tongues";
import Modal from "./Modal";

interface Props {
  doc: string;
  word: LanguageWord;
  onDone: () => void;
  onClose: () => void;
  onError?: (problem: unknown) => void;
}

const codeOf = (problem: unknown) => (problem as { code?: string } | null)?.code;

type Said = NonNullable<LanguageShown["changes"]>;

const blockTone = "mt-2 block whitespace-pre-wrap text-[12.5px] leading-relaxed first:mt-0";

const toneOf = {
  same: "text-ink",
  gone: "text-faint line-through decoration-faint",
  came: "rounded-md bg-hue-green/10 px-1 text-ink no-underline",
} as const;

const tagOf = { same: "p", gone: "del", came: "ins" } as const;

function Changes({ said }: { said: Said }) {
  return (
    <div>
      {said.map((one, at) => {
        const Tag = tagOf[one.kind];
        return (
          // biome-ignore lint/suspicious/noArrayIndexKey: what changed is told once and never moves
          <Tag key={at} className={`${blockTone} ${toneOf[one.kind]}`}>
            {one.text}
          </Tag>
        );
      })}
    </div>
  );
}

export default function LanguageReview({ doc, word, onDone, onClose, onError }: Props) {
  const [shown, setShown] = useState<LanguageShown | null>(null);
  const [told, setTold] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  // The window hands new callbacks on each render, and only a new language or a new look asks Folio again.
  const latest = useRef({ onClose, onError });
  latest.current = { onClose, onError };
  const open = useRef(true);

  useEffect(
    () => () => {
      open.current = false;
    },
    [],
  );

  const bring = useCallback(() => {
    setShown(null);
    languageShown(doc, word.document)
      .then((now) => {
        if (open.current) setShown(now);
      })
      .catch((problem) => {
        if (!open.current) return;
        latest.current.onError?.(problem);
        latest.current.onClose();
      });
  }, [doc, word.document]);

  useEffect(() => {
    bring();
  }, [bring]);

  const decide = (keep: boolean) => {
    setBusy(true);
    setTold(null);
    languageReviewed(doc, word.document, keep, keep ? (shown?.print ?? null) : null)
      .then(onDone)
      .catch((problem) => {
        if (codeOf(problem) === "folioLanguageChanged") {
          setTold(t("folioLanguageChanged"));
          bring();
          return;
        }
        onError?.(problem);
      })
      .finally(() => setBusy(false));
  };

  const name = tongueName(word.lang ?? shown?.lang ?? "");
  const changed = word.word === "changed";
  return (
    <Modal
      title={fill(changed ? "reviewChangesTitle" : "reviewTitle", name)}
      wide
      onClose={onClose}
    >
      {shown?.by && <p className="mt-2 text-[12.5px] text-soft">{fill("reviewBy", shown.by)}</p>}
      <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
        {t(changed ? "reviewChangesWhy" : "reviewWhy")}
      </p>
      {changed && shown && !shown.changes && (
        <p className="mt-2 text-[12.5px] text-faint">{t("reviewNoCopy")}</p>
      )}
      {told && (
        <p role="status" className="mt-3 text-[12.5px] text-ink">
          {told}
        </p>
      )}
      {shown === null ? (
        <p className="mt-4 text-[12.5px] text-faint">{t("reviewBringing")}</p>
      ) : (
        <div className="scroller mt-4 max-h-[50vh] rounded-[10px] border border-hair p-3">
          {shown.changes ? (
            <Changes said={shown.changes} />
          ) : (
            <>
              {shown.title && <p className="text-[13px] font-medium text-ink">{shown.title}</p>}
              <p className="mt-2 whitespace-pre-wrap text-[12.5px] leading-relaxed text-ink">
                {shown.body}
              </p>
            </>
          )}
          {shown.pictures.map((one) => (
            <img
              key={one.at}
              alt={fill("reviewPicture", String(one.at + 1))}
              src={convertFileSrc(one.path)}
              className="mt-3 max-w-full rounded-[10px]"
            />
          ))}
        </div>
      )}
      <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
        <button
          type="button"
          onClick={onClose}
          className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
        >
          {t("cancel")}
        </button>
        <button
          type="button"
          disabled={busy}
          onClick={() => decide(false)}
          className="cursor-pointer rounded-[10px] border border-line px-3 py-1.5 text-ink hover:bg-line/40 disabled:opacity-50"
        >
          {t("reviewLeaveOut")}
        </button>
        <button
          type="button"
          disabled={busy || shown === null}
          onClick={() => decide(true)}
          className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg disabled:opacity-50"
        >
          {t("reviewKeep")}
        </button>
      </div>
    </Modal>
  );
}
