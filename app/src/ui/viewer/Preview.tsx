import type { Paper } from "../../core";
import { t } from "../../locales";
import { leafWord } from "../sheets";
import { button, tinted } from "./Glyphs";

interface Props {
  shown: string;
  leaf: Paper;
  signing: boolean | null;
  making: boolean;
  onSign: (signed: boolean) => void;
  onKeep: () => void;
  onClose: () => void;
}

export default function Preview({ shown, leaf, signing, making, onSign, onKeep, onClose }: Props) {
  return (
    <div className="absolute inset-0 z-30 flex flex-col bg-desk">
      <div className="flex h-10 shrink-0 items-center gap-2 border-b border-hair bg-panel pr-2.5 pl-4 text-[12.5px]">
        <b className="font-semibold">{t("seePdf")}</b>
        <span className="text-faint">· {t(leafWord(leaf))}</span>
        <div className="flex-1" />
        {signing !== null && (
          <label className="mr-1 flex items-center gap-1.5 text-[12.5px] text-soft">
            <input
              type="checkbox"
              checked={signing}
              disabled={making}
              onChange={(e) => onSign(e.target.checked)}
              className="h-3.5 w-3.5 accent-accent"
            />
            {t("viewWithSign")}
          </label>
        )}
        <button type="button" className={tinted} disabled={making} onClick={onKeep}>
          {t("toPdf")}
        </button>
        <button
          type="button"
          className={`${button} border-transparent text-soft`}
          onClick={onClose}
        >
          {t("viewClose")}
        </button>
      </div>
      <iframe src={shown} title={t("seePdf")} className="min-h-0 flex-1 border-0 bg-bg" />
    </div>
  );
}
