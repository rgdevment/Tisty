import { useState } from "react";
import type { Filed } from "../core";
import { fill, t } from "../locales";
import { tongueName } from "../tongues";
import Glyph from "./Glyph";
import Menu from "./Menu";

const SIDE_BY_SIDE = 3;
const FIND_FROM = 9;

interface Props {
  tongues: Filed[];
  here: string;
  onOpen: (tongue: Filed) => void;
}

// Each language arrives under its root's name and only one its anchor's owner kept reaches a reader, so another name means that owner reviewed it (166).
export const creditOf = (one: Filed | undefined, tongues: Filed[]): string | null => {
  const anchor = tongues.find((it) => it.id === one?.tongue?.[0]);
  const by = one?.guest;
  const owner = anchor?.guest;
  return by && owner && by !== owner ? fill("tongueCredit", by, owner) : null;
};

const titled = (one: Filed, tongues: Filed[]) =>
  [one.title, creditOf(one, tongues)].filter(Boolean).join(" — ");

function Credit({ tongues, here }: { tongues: Filed[]; here: string }) {
  const said = creditOf(
    tongues.find((one) => one.id === here),
    tongues,
  );
  return said ? <span className="self-center px-1 text-[11.5px] text-faint">{said}</span> : null;
}

export default function Tongues({ tongues, here, onOpen }: Props) {
  if (tongues.filter((one) => !one.away).length > SIDE_BY_SIDE)
    return <Folded tongues={tongues} here={here} onOpen={onOpen} />;
  return (
    <fieldset className="tongues">
      <legend className="sr-only">{t("tonguesOf")}</legend>
      <Credit tongues={tongues} here={here} />
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
            title={one.away ? `${tongueName(lang)} — ${t("isArchived")}` : titled(one, tongues)}
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
      <Credit tongues={tongues} here={here} />
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={at !== null}
        aria-label={`${t("tonguesOf")} · ${said} · ${live.length}`}
        onClick={(e) => {
          const box = e.currentTarget.getBoundingClientRect();
          setAt({ x: box.left, y: box.bottom + 4 });
        }}
        className="flex items-center gap-2 rounded-md border border-line px-2.5 py-1 text-ink hover:bg-hover"
      >
        <Glyph name="language" className="h-3.5 w-3.5 text-faint" />
        <span lang={shown?.lang ?? undefined}>{said}</span>
        <span className="text-faint tabular-nums">{live.length}</span>
        <Glyph name="chevron" className="h-3 w-3 text-faint" />
      </button>
      {at && (
        <Menu
          at={at}
          label={t("tonguesOf")}
          find={live.length >= FIND_FROM ? t("tonguesFind") : undefined}
          choices={live.map((one) => ({
            key: one.id,
            code: (one.lang ?? "").toUpperCase(),
            on: one.id === here,
            label: tongueName(one.lang ?? ""),
            hint: titled(one, tongues),
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
