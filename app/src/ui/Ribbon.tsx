import type { Filed } from "../core";
import { fill, t } from "../locales";

interface Props {
  of: Filed;
  sisters: Filed[];
  told?: Set<string>;
  here: string;
  onOpen: (doc: Filed) => void;
}

export default function Ribbon({ of, sisters, told, here, onOpen }: Props) {
  const read = told ? sisters.filter((one) => told.has(one.file)) : [];
  const at = told ? read.findIndex((one) => one.file === here) : -1;
  const back = at > 0 ? read[at - 1] : undefined;
  const on = at >= 0 && at + 1 < read.length ? read[at + 1] : undefined;

  return (
    <nav aria-label={t("whereThisSits")} className="ribbon">
      <button type="button" onClick={() => onOpen(of)} className="ribbon-back">
        <span aria-hidden="true">‹</span>
        <span className="truncate">{of.title || t("untitledDoc")}</span>
      </button>
      {told && (
        <span className="ribbon-at">
          {at >= 0 ? fill("leafOfMany", String(at + 1), String(read.length)) : t("looseLeafIs")}
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
