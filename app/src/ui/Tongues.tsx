import type { Filed } from "../core";
import { t } from "../locales";
import { tongueName } from "../tongues";

interface Props {
  tongues: Filed[];
  here: string;
  onOpen: (tongue: Filed) => void;
}

export default function Tongues({ tongues, here, onOpen }: Props) {
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
