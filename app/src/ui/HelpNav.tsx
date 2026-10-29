import type { Head } from "tisty-reader/page";
import { openLink } from "../core";
import type { HelpEntry } from "../guide/calls";
import { t } from "../locales";

interface Props {
  readonly sections: [string, HelpEntry[]][];
  readonly marks: HelpEntry[];
  readonly here: string | null;
  readonly subs: Head[];
  readonly onHome: () => void;
  readonly onOpen: (slug: string) => void;
}

const item = (here: boolean) =>
  `w-full rounded-md px-2.5 py-1.5 text-left text-[13px] leading-snug ${
    here ? "bg-accent-soft font-semibold text-accent" : "text-ink hover:bg-hover"
  }`;

const heading =
  "mx-2.5 mt-3 border-t border-hair pt-3 pb-1 text-[10.5px] font-semibold tracking-[0.08em] text-faint uppercase";

export default function HelpNav({ sections, marks, here, subs, onHome, onOpen }: Props) {
  const listed = (pages: HelpEntry[]) => (
    <ol className="flex flex-col gap-px">
      {pages.map((one) => {
        const shown = here === one.slug;
        return (
          <li key={one.slug}>
            <button
              type="button"
              aria-current={shown ? "page" : undefined}
              onClick={() => onOpen(one.slug)}
              className={item(shown)}
            >
              {one.title}
            </button>
            {shown && subs.length > 0 && (
              <ol className="my-0.5 ml-3 border-l border-hair pl-2">
                {subs.map((sub) => (
                  <li key={sub.key}>
                    <button
                      type="button"
                      onClick={sub.go}
                      className="w-full rounded-md px-2 py-1 text-left text-[12.5px] text-faint hover:bg-hover hover:text-ink"
                    >
                      {sub.text}
                    </button>
                  </li>
                ))}
              </ol>
            )}
          </li>
        );
      })}
    </ol>
  );

  return (
    <nav
      aria-label={t("helpTopics")}
      className="scroller flex w-[260px] shrink-0 flex-col overflow-y-auto px-3 pb-6"
    >
      <button
        type="button"
        aria-current={here === null ? "page" : undefined}
        onClick={onHome}
        className={item(here === null)}
      >
        {t("helpStart")}
      </button>
      {sections.map(([section, pages]) => (
        <div key={section}>
          <p className={heading}>{section}</p>
          {listed(pages)}
        </div>
      ))}
      {marks.length > 0 && (
        <div>
          <p className={heading}>Markdown</p>
          {listed(marks)}
        </div>
      )}
      <button
        type="button"
        onClick={() => openLink(t("helpMoreAt")).catch(() => undefined)}
        className="mt-6 px-2.5 text-left text-[11.5px] text-faint hover:text-soft hover:underline"
      >
        {t("helpMore")} ↗
      </button>
    </nav>
  );
}
