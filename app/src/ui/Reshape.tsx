import { createPortal } from "react-dom";
import { t } from "../locales";
import type { Shape } from "../reshaping";
import Modal from "./Modal";

interface Props {
  said: string;
  onAnswer: (shape: Shape | undefined) => void;
}

export default function Reshape({ said, onAnswer }: Props) {
  // At the end of the body, so a question asked from inside another window is drawn above it.
  return createPortal(
    <Modal title={t("reshapeTitle")} onClose={() => onAnswer(undefined)}>
      <p className="px-1 text-[13px] leading-relaxed whitespace-pre-line text-soft">{said}</p>
      <div className="mt-4 flex flex-wrap justify-end gap-2">
        <button
          type="button"
          onClick={() => onAnswer(undefined)}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
        >
          {t("reshapeKeep")}
        </button>
        <button
          type="button"
          onClick={() => onAnswer("light")}
          className="rounded-md border border-line px-3 py-1 text-[12.5px] hover:border-ink"
        >
          {t("reshapeLight")}
        </button>
        <button
          type="button"
          onClick={() => onAnswer("full")}
          className="rounded-md bg-accent px-3 py-1 text-[12.5px] text-bg"
        >
          {t("reshapeFull")}
        </button>
      </div>
    </Modal>,
    document.body,
  );
}
