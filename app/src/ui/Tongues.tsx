import { useState } from "react";
import type { Filed } from "../core";
import { t } from "../locales";
import { tongueName } from "../tongues";
import Glyph from "./Glyph";
import Menu from "./Menu";

const SIDE_BY_SIDE = 3;

interface Props {
  tongues: Filed[];
  here: string;
  onOpen: (tongue: Filed) => void;
}

export default function Tongues({ tongues, here, onOpen }: Props) {
  if (tongues.length > SIDE_BY_SIDE)
    return <Folded tongues={tongues} here={here} onOpen={onOpen} />;
  return (
    <fieldset className="tongues">
      <legend className="sr-only">{t("tonguesOf")}</legend>
      {tongues.map((one) => {
        const lang = one.lang ?? "";
        const on = one.id === here;
        return (
          <button
            key={one.id}
            type="button"
            lang={lang}
            aria-pressed={on}
            disabled={one.away}
            title={one.away ? `${tongueName(lang)} — ${t("isArchived")}` : one.title}
            onClick={() => !on && onOpen(one)}
            className={on ? "tongue on" : "tongue"}
          >
            {tongueName(lang)}
          </button>
        );
      })}
    </fieldset>
  );
}

function Folded({ tongues, here, onOpen }: Props) {
  const [at, setAt] = useState<{ x: number; y: number } | null>(null);
  const shown = tongues.find((one) => one.id === here);
  const said = tongueName(shown?.lang ?? "");
  const live = tongues.filter((one) => !one.away);
  return (
    <div className="tongues">
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={at !== null}
        aria-label={`${t("tonguesOf")} · ${said} · ${live.length}`}
        onClick={(e) => {
          const box = e.currentTarget.getBoundingClientRect();
          setAt({ x: box.left, y: box.bottom + 4 });
        }}
        className="tongue on flex items-center gap-1.5"
      >
        <span lang={shown?.lang ?? undefined}>{said}</span>
        <span className="font-normal text-faint tabular-nums">{live.length}</span>
        <Glyph name="chevron" className="h-3 w-3 text-faint" />
      </button>
      {at && (
        <Menu
          at={at}
          label={t("tonguesOf")}
          choices={live.map((one) => ({
            key: one.id,
            icon: one.id === here ? "✓" : "",
            label: tongueName(one.lang ?? ""),
            hint: one.title,
            onPick: () => one.id !== here && onOpen(one),
          }))}
          onClose={() => setAt(null)}
        />
      )}
    </div>
  );
}

/// The heading the reader stands under, counted as the index counts them, for another language.
export const standingHead = (): number => {
  const sheet = document.querySelector<HTMLElement>(".scroller.gutter");
  const frame = sheet?.querySelector<HTMLIFrameElement>("iframe");
  const page = frame?.contentDocument;
  if (!sheet || !frame || !page) return -1;
  const top = sheet.getBoundingClientRect().top - frame.getBoundingClientRect().top + 24;
  const marks = page.querySelectorAll<HTMLElement>(
    ".page > :is(h1, h2, h3, h4, h5, h6):not([data-title])",
  );
  let at = -1;
  marks.forEach((one, n) => {
    if (one.getBoundingClientRect().top <= top) at = n;
  });
  return at;
};
