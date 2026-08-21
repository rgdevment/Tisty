import { useEffect, useState } from "react";
import { icons as readIcons } from "../core";
import { t } from "../locales";

let held: [string, string][] | null = null;

export function useIcons() {
  const [all, setAll] = useState<[string, string][]>(held ?? []);

  useEffect(() => {
    if (held) return;
    readIcons()
      .then((found) => {
        held = found;
        setAll(found);
      })
      .catch(() => {});
  }, []);

  return all;
}

export function drawn(all: [string, string][], key?: string): string | null {
  if (!key) return null;
  return all.find(([named]) => named === key)?.[1] ?? null;
}

interface Props {
  chosen?: string;
  onPick: (key: string | undefined) => void;
}

export const sifted = (all: [string, string][], word: string): [string, string][] => {
  const said = word.trim().toLowerCase();
  if (!said) return all;
  return all.filter(([key]) => key.includes(said));
};

export default function Icons({ chosen, onPick }: Props) {
  const all = useIcons();
  const [word, setWord] = useState("");
  const shown = sifted(all, word);

  return (
    <div className="flex flex-col gap-1.5">
      <input
        value={word}
        onChange={(e) => setWord(e.target.value)}
        placeholder={t("siftIcons")}
        aria-label={t("siftIcons")}
        className="w-full rounded-lg bg-hover px-2.5 py-1 text-[12px] outline-none placeholder:text-faint"
      />
      <fieldset className="flex flex-wrap gap-1">
        <legend className="sr-only">{t("pickAnIcon")}</legend>
        <button
          type="button"
          onClick={() => onPick(undefined)}
          aria-pressed={!chosen}
          aria-label={t("noIcon")}
          title={t("noIcon")}
          className={`grid h-8 w-8 place-items-center rounded-lg text-[13px] text-faint ${
            chosen ? "hover:bg-hover" : "bg-accent-soft text-accent"
          }`}
        >
          ○
        </button>
        {shown.map(([key, glyph]) => (
          <button
            key={key}
            type="button"
            onClick={() => onPick(key)}
            aria-pressed={chosen === key}
            aria-label={key}
            title={key}
            className={`grid h-8 w-8 place-items-center rounded-lg text-[16px] ${
              chosen === key ? "bg-accent-soft" : "hover:bg-hover"
            }`}
          >
            {glyph}
          </button>
        ))}
      </fieldset>
    </div>
  );
}
