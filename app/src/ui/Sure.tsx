import { useRef } from "react";
import { createPortal } from "react-dom";
import { t } from "../locales";
import type { Buttons } from "../sure";
import Modal from "./Modal";

/// Long enough that the second click of a double-click lands before a question queued behind it can take it.
export const SETTLING_MS = 400;

interface Props {
  said: string;
  buttons?: Buttons;
  hurried?: boolean;
  onAnswer: (yes: boolean) => void;
}

export default function Sure({ said, buttons, hurried, onAnswer }: Props) {
  const opened = useRef(performance.now());
  const yes = useRef<HTMLButtonElement>(null);
  const answer = (yes: boolean) => {
    if (yes && hurried && performance.now() - opened.current < SETTLING_MS) return;
    onAnswer(yes);
  };
  // At the end of the body, so a question asked from inside another window is drawn above it.
  return createPortal(
    // Enter goes on, as the question asks; an Enter still held from the key that asked it goes nowhere.
    <Modal title={t("sureTitle")} first={yes} onClose={() => answer(false)}>
      <p className="px-1 text-[13px] leading-relaxed whitespace-pre-line text-soft">{said}</p>
      <div className="mt-4 flex justify-end gap-2">
        <button
          type="button"
          onClick={() => answer(false)}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
        >
          {buttons?.no ?? t("cancel")}
        </button>
        <button
          ref={yes}
          type="button"
          onKeyDown={(e) => {
            if (e.key === "Enter" && e.repeat) e.preventDefault();
          }}
          onClick={() => answer(true)}
          className="rounded-md border border-line px-3 py-1 text-[12.5px] hover:border-ink"
        >
          {buttons?.yes ?? t("sureGoOn")}
        </button>
      </div>
    </Modal>,
    document.body,
  );
}
