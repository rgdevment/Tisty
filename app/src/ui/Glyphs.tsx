import { useState } from "react";
import { useIcons } from "./Icons";
import { t } from "../locales";

export default function Glyphs({ onPick }: { onPick: (glyph: string) => void }) {
  const all = useIcons();
  const [word, setWord] = useState("");

  const shown = all.filter(([named]) => named.includes(word.trim().toLowerCase()));

  return (
    <>
      <input
        autoFocus
        value={word}
        aria-label={t("pickAnIcon")}
        placeholder={t("pickAnIcon")}
        onChange={(e) => setWord(e.target.value)}
        className="mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 text-[12.5px] outline-none placeholder:text-faint"
      />
      <div role="group" aria-label={t("pickAnIcon")} className="scroller max-h-[196px]">
        {shown.length === 0 && <p className="px-2.5 py-1.5 text-faint">{t("noneHere")}</p>}
        <div className="flex flex-wrap gap-0.5">
          {shown.map(([named, glyph]) => (
            <button
              key={named}
              type="button"
              title={named}
              aria-label={named}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => onPick(glyph)}
              className="grid h-8 w-8 place-items-center rounded-md text-[17px] hover:bg-hover"
            >
              {glyph}
            </button>
          ))}
        </div>
      </div>
    </>
  );
}
