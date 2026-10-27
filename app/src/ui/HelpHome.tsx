import { useMemo, useState } from "react";
import type { HelpEntry } from "../guide/calls";
import { describedBy, searched, sectionsOf, suggestedFor } from "../guide/help";
import { fill, t } from "../locales";

interface Props {
  readonly entries: HelpEntry[];
  readonly more: HelpEntry[];
  readonly bodies: Record<string, string>;
  readonly from?: string;
  readonly spanish: boolean;
  readonly onOpen: (slug: string, heading?: string) => void;
  readonly onMarkdown: () => void;
}

const card =
  "flex flex-col gap-1.5 rounded-[10px] border border-line bg-sheet p-4 text-left hover:border-accent";

const title = "text-[11.5px] font-semibold tracking-wide text-faint uppercase";

export default function HelpHome({
  entries,
  more,
  bodies,
  from,
  spanish,
  onOpen,
  onMarkdown,
}: Props) {
  const [asked, setAsked] = useState("");
  const found = useMemo(
    () => searched([...entries, ...more], bodies, asked),
    [entries, more, bodies, asked],
  );
  const named = (slug: string) => entries.find((one) => one.slug === slug)?.title;
  const here = suggestedFor(from, spanish).filter((one) => named(one.slug));

  return (
    <div className="scroller min-h-0 flex-1 overflow-y-auto px-8 pb-10">
      <div className="mx-auto flex max-w-[860px] flex-col gap-7 pt-4">
        <header className="flex flex-col gap-2.5">
          <h1 className="text-[21px] font-semibold text-ink">{t("helpAsk")}</h1>
          <label htmlFor="help-search" className="text-[12.5px] text-soft">
            {t("helpSearch")}
          </label>
          <input
            id="help-search"
            type="search"
            value={asked}
            onChange={(event) => setAsked(event.target.value)}
            placeholder={t("helpSearchHint")}
            className="w-full rounded-[10px] border border-line bg-sheet px-4 py-3 text-[13px] text-ink outline-none focus:border-accent"
          />
        </header>

        {asked.trim() ? (
          <section aria-label={t("helpSearch")} className="flex flex-col gap-2">
            {found.length === 0 && (
              <p className="text-[13px] text-soft">{fill("helpNothing", asked.trim())}</p>
            )}
            {found.map(({ entry, near }) => (
              <button
                key={entry.slug}
                type="button"
                onClick={() => onOpen(entry.slug)}
                className={card}
              >
                <span className="text-[13px] font-semibold text-ink">{entry.title}</span>
                <span className="text-[11.5px] text-faint">{entry.section}</span>
                {near && <span className="text-[12.5px] leading-relaxed text-soft">…{near}…</span>}
              </button>
            ))}
          </section>
        ) : (
          <>
            {here.length > 0 && (
              <section aria-labelledby="help-here" className="flex flex-col gap-3">
                <h2 id="help-here" className={title}>
                  {t("helpHere")}
                </h2>
                <div className="flex flex-wrap gap-2">
                  {here.map((one) => (
                    <button
                      key={`${one.slug}:${one.heading ?? ""}`}
                      type="button"
                      onClick={() => onOpen(one.slug, one.heading)}
                      className="rounded-full border border-line px-3.5 py-1.5 text-[13px] text-ink hover:border-accent hover:text-accent"
                    >
                      {one.heading ?? named(one.slug)}
                    </button>
                  ))}
                </div>
              </section>
            )}
            {sectionsOf(entries).map(([section, pages]) => (
              <section key={section} aria-label={section} className="flex flex-col gap-3">
                <h2 className={title}>{section}</h2>
                <div className="grid grid-cols-[repeat(auto-fill,minmax(230px,1fr))] gap-3">
                  {pages.map((one) => (
                    <button
                      key={one.slug}
                      type="button"
                      onClick={() => onOpen(one.slug)}
                      className={card}
                    >
                      <span className="text-[13px] font-semibold text-ink">{one.title}</span>
                      <span className="line-clamp-2 text-[12.5px] leading-relaxed text-soft">
                        {describedBy(bodies[one.slug] ?? "")}
                      </span>
                    </button>
                  ))}
                </div>
              </section>
            ))}
            <section aria-label="Markdown" className="flex flex-col gap-3">
              <h2 className={title}>Markdown</h2>
              <button type="button" onClick={onMarkdown} className={`${card} max-w-[320px]`}>
                <span className="text-[13px] font-semibold text-ink">{t("helpMarkdown")}</span>
                <span className="text-[12.5px] leading-relaxed text-soft">
                  {t("helpMarkdownWhy")}
                </span>
              </button>
            </section>
          </>
        )}
      </div>
    </div>
  );
}
