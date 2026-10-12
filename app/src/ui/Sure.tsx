import { t } from "../locales";
import Modal from "./Modal";

interface Props {
  said: string;
  onAnswer: (yes: boolean) => void;
}

export default function Sure({ said, onAnswer }: Props) {
  return (
    <Modal title={t("sureTitle")} onClose={() => onAnswer(false)}>
      <p className="px-1 text-[13px] leading-relaxed whitespace-pre-line text-soft">{said}</p>
      <div className="mt-4 flex justify-end gap-2">
        <button
          type="button"
          onClick={() => onAnswer(false)}
          className="rounded-md px-2.5 py-1 text-[12.5px] text-faint hover:bg-hover"
        >
          {t("cancel")}
        </button>
        <button
          type="button"
          onClick={() => onAnswer(true)}
          className="rounded-md border border-line px-3 py-1 text-[12.5px] hover:border-ink"
        >
          {t("sureGoOn")}
        </button>
      </div>
    </Modal>
  );
}
