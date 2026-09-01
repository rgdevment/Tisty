import type { Filed } from "../core";
import { fill, t } from "../locales";

interface Props {
  of: Filed;
  sisters: Filed[];
  here: string;
  onOpen: (doc: Filed) => void;
}

export default function Ribbon({ of, sisters, here, onOpen }: Props) {
  const at = sisters.findIndex((one) => one.file === here);
  const back = at > 0 ? sisters[at - 1] : undefined;
  const on = at >= 0 && at + 1 < sisters.length ? sisters[at + 1] : undefined;

  return (
    <nav aria-label={t("whereThisSits")} className="ribbon">
      <button type="button" onClick={() => onOpen(of)} className="ribbon-back">
        <span aria-hidden="true">‹</span>
        <span className="truncate">{of.title || t("untitledDoc")}</span>
      </button>
      {at >= 0 && (
        <span className="ribbon-at">
          {fill("leafOfMany", String(at + 1), String(sisters.length))}
        </span>
      )}
      <span className="ribbon-arrows">
        <button
          type="button"
          disabled={!back}
          onClick={() => back && onOpen(back)}
          title={back ? back.title || t("untitledDoc") : t("noLeafBack")}
          aria-label={t("leafBack")}
        >
          <span aria-hidden="true">‹</span>
        </button>
        <button
          type="button"
          disabled={!on}
          onClick={() => on && onOpen(on)}
          title={on ? on.title || t("untitledDoc") : t("noLeafOn")}
          aria-label={t("leafOn")}
        >
          <span aria-hidden="true">›</span>
        </button>
      </span>
    </nav>
  );
}

export function Onward({ next, onOpen }: { next: Filed; onOpen: (doc: Filed) => void }) {
  return (
    <button type="button" onClick={() => onOpen(next)} className="onward">
      <span className="onward-say">{t("leafNext")}</span>
      <span className="onward-name">{next.title || t("untitledDoc")}</span>
      <span aria-hidden="true" className="onward-go">
        ›
      </span>
    </button>
  );
}
