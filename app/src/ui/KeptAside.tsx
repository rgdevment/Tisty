import { type CSSProperties, useState } from "react";
import { copied } from "../core";
import { letAsideGo, useAside } from "../keptAside";
import { t } from "../locales";
import { sure } from "../sure";
import Modal from "./Modal";
import { BAR } from "./SaveBars";
import { MILD } from "./Tidied";

interface Props {
  file: string;
  style: CSSProperties;
  writable: boolean;
  onPutBack: (words: string) => void;
  onError: (problem: unknown) => void;
}

export default function KeptAside({ file, style, writable, onPutBack, onError }: Readonly<Props>) {
  const words = useAside(file);
  const [shown, setShown] = useState(false);
  const [copiedNow, setCopiedNow] = useState(false);
  if (words === undefined) return null;
  const discard = async () => {
    if (!(await sure(t("asideDiscardSure")))) return;
    setShown(false);
    letAsideGo(file);
  };
  const putBack = () => {
    setShown(false);
    letAsideGo(file);
    onPutBack(words);
  };
  return (
    <>
      <div role="status" style={style} className={BAR}>
        <span className="text-soft">{t("asideKept")}</span>
        <button type="button" onClick={() => setShown(true)} className={MILD}>
          {t("asideRecover")}
        </button>
        <button type="button" onClick={() => void discard()} className={MILD}>
          {t("asideDiscard")}
        </button>
      </div>
      {shown && (
        <Modal title={t("asideTitle")} wide onClose={() => setShown(false)}>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">
            {t(writable ? "asideWhy" : "asideOnlyCopy")}
          </p>
          <textarea
            aria-label={t("asideTitle")}
            readOnly
            value={words}
            className="mt-3 h-64 w-full resize-none rounded-[10px] border border-line bg-bg p-3 text-[12.5px] text-ink"
          />
          <div className="mt-5 flex items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => setShown(false)}
              className="rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("close")}
            </button>
            <button
              type="button"
              onClick={() =>
                void copied(words)
                  .then(() => setCopiedNow(true))
                  .catch(onError)
              }
              className="rounded-[10px] border border-line px-3 py-1.5 hover:bg-hover"
            >
              {t(copiedNow ? "asideCopied" : "copyIt")}
            </button>
            {writable && (
              <button
                type="button"
                onClick={putBack}
                className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg"
              >
                {t("asidePutBack")}
              </button>
            )}
          </div>
        </Modal>
      )}
    </>
  );
}
