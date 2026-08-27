import { useState } from "react";
import { known } from "../glyphs";
import { t } from "../locales";
import Glyph from "./Glyph";
import Hue from "./Hue";
import { sifted, useIcons } from "./Icons";

interface Props {
  icon?: string;
  colour?: string | null;
  onIcon: (key: string | undefined) => void;
  onColour?: (hue: string | undefined) => void;
  autoFocus?: boolean;
  keepFocus?: boolean;
  clears?: boolean;
  tall?: string;
}

export default function Pick({
  icon,
  colour,
  onIcon,
  onColour,
  autoFocus,
  keepFocus,
  clears = true,
  tall = "max-h-52",
}: Props) {
  const all = useIcons();
  const [word, setWord] = useState("");
  const shown = sifted(all, word).filter(known);
  const hold = keepFocus ? (e: React.MouseEvent) => e.preventDefault() : undefined;

  return (
    <div className="flex min-h-0 flex-col gap-1.5">
      <input
        autoFocus={autoFocus}
        value={word}
        onChange={(e) => setWord(e.target.value)}
        placeholder={t("siftIcons")}
        aria-label={t("siftIcons")}
        className="w-full shrink-0 rounded-lg bg-hover px-2.5 py-1 text-[12px] outline-none placeholder:text-faint"
      />
      <fieldset
        className={`scroller flex min-h-0 flex-wrap content-start gap-1 overflow-y-auto ${tall}`}
      >
        <legend className="sr-only">{t("pickAnIcon")}</legend>
        {clears && (
          <button
            type="button"
            onMouseDown={hold}
            onClick={() => onIcon(undefined)}
            aria-pressed={!icon}
            aria-label={t("noIcon")}
            title={t("noIcon")}
            className={`grid h-8 w-8 place-items-center rounded-lg text-[13px] text-faint ${
              icon ? "hover:bg-hover" : "bg-accent-soft text-accent"
            }`}
          >
            ○
          </button>
        )}
        {shown.length === 0 && (
          <p className="px-2.5 py-1.5 text-[12px] text-faint">{t("noneHere")}</p>
        )}
        {shown.map((key) => (
          <button
            key={key}
            type="button"
            onMouseDown={hold}
            onClick={() => onIcon(key)}
            aria-pressed={icon === key}
            aria-label={key}
            title={key}
            className={`grid h-8 w-8 place-items-center rounded-lg ${
              icon === key ? "bg-accent-soft" : "hover:bg-hover"
            }`}
          >
            <Glyph name={key} />
          </button>
        ))}
      </fieldset>
      {onColour && (
        <div className="shrink-0 border-t border-hair pt-2">
          <Hue chosen={colour} onPick={onColour} onHold={hold} />
        </div>
      )}
    </div>
  );
}
