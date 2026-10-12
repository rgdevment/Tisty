import { useRef } from "react";
import { createPortal } from "react-dom";
import { t } from "../locales";
import Modal from "./Modal";

/// Long enough that the second click of a double-click lands before a question queued behind it can take it.
export const SETTLING_MS = 400;

interface Props {
  said: string;
  hurried?: boolean;
  onAnswer: (yes: boolean) => void;
}

export default function Sure({ said, hurried, onAnswer }: Props) {
  const opened = useRef(performance.now());
  const answer = (yes: boolean) => {
    if (yes && hurried && performance.now() - opened.current < SETTLING_MS) return;
    onAnswer(yes);
  };
  // At the end of the body, so a question asked from inside another window is drawn above it.
  return createPortal(
    <Modal title={t("sureTitle")} onClose={() => answer(false)}>
      <p className="px-1 text-[13px] leading-relaxed whitespace-pre-line text-soft">{said}</p>
      <div className="mt-4 flex justify-end gap-2">
        <button
          type="button"
          onClick={() => answer(false)}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
        >
          {t("cancel")}
        </button>
        <button
          type="button"
          onClick={() => answer(true)}
          className="rounded-md border border-line px-3 py-1 text-[12.5px] hover:border-ink"
        >
          {t("sureGoOn")}
        </button>
      </div>
    </Modal>,
    document.body,
  );
}
