import type { Paper } from "../../core";
import { t } from "../../locales";
import PdfTick from "../PdfTick";
import { leafWord } from "../sheets";
import { clearOfChrome, onMac } from "../WindowChrome";
import { button, tinted } from "./Glyphs";

interface Props {
  shown: string;
  leaf: Paper;
  carding: boolean | null;
  crediting: boolean;
  making: boolean;
  onCard: (on: boolean) => void;
  onCredit: (on: boolean) => void;
  onKeep: () => void;
  onClose: () => void;
}

export default function Preview({
  shown,
  leaf,
  carding,
  crediting,
  making,
  onCard,
  onCredit,
  onKeep,
  onClose,
}: Props) {
  return (
    <div className="absolute inset-0 z-30 flex flex-col bg-desk">
      <div
        data-tauri-drag-region
        className={`flex h-11 shrink-0 items-center gap-2 border-b border-hair bg-panel text-[12.5px] ${onMac ? "pl-20" : "pl-4"} ${clearOfChrome}`}
      >
        <button
          type="button"
          className={`${button} -ml-1.5 border-transparent text-soft`}
          onClick={onClose}
        >
          {t("previewBack")}
        </button>
        <b data-tauri-drag-region className="font-semibold">
          {t("seePdf")}
        </b>
        <span data-tauri-drag-region className="text-faint">
          · {t(leafWord(leaf))}
        </span>
        <div data-tauri-drag-region className="flex-1" />
        {carding !== null && (
          <PdfTick
            barred
            on={carding}
            disabled={making}
            label={t("pdfSign")}
            why={t("pdfSignAsk")}
            onTick={onCard}
          />
        )}
        <PdfTick
          barred
          on={crediting}
          disabled={making}
          label={t("pdfCreditTick")}
          why={t("pdfCreditAsk")}
          onTick={onCredit}
        />
        <button type="button" className={tinted} disabled={making} onClick={onKeep}>
          {t("toPdf")}
        </button>
      </div>
      <iframe src={shown} title={t("seePdf")} className="min-h-0 flex-1 border-0 bg-bg" />
    </div>
  );
}
