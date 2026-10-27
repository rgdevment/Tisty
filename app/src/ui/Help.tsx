import { useEffect, useState } from "react";
import { openLink } from "../core";
import { weigh } from "../format";
import {
  type HelpEntry,
  type HelpPictures,
  helpPage,
  helpPages,
  helpPictures,
  helpPicturesFetch,
  helpReferenceHere,
} from "../guide/calls";
import { forTheWindow, MARKS, marksOf, sectionsOf } from "../guide/help";
import { topicsOf } from "../guide/reference";
import { fill, locale, t } from "../locales";
import { saidPlainly } from "../refusal";
import HelpArticle from "./HelpArticle";
import HelpHome from "./HelpHome";
import MarkdownGuide from "./MarkdownGuide";

type View =
  | { kind: "home" }
  | { kind: "page"; slug: string; heading?: string; anchor?: string | null }
  | { kind: "markdown"; topic?: string };

const item = (here: boolean) =>
  `w-full rounded-md px-2.5 py-1.5 text-left text-[13px] leading-snug ${
    here ? "bg-accent-soft font-semibold text-accent" : "text-ink hover:bg-hover"
  }`;

const heading =
  "mx-2.5 mt-3 border-t border-hair pt-3 pb-1 text-[10.5px] font-semibold tracking-[0.08em] text-faint uppercase";

export default function Help({ topic, from }: { readonly topic?: string; readonly from?: string }) {
  const spanish = locale().toLowerCase().startsWith("es");
  const [view, setView] = useState<View>(topic ? { kind: "markdown", topic } : { kind: "home" });
  const [entries, setEntries] = useState<HelpEntry[]>([]);
  const [bodies, setBodies] = useState<Record<string, string>>({});
  const [marks, setMarks] = useState<HelpEntry[]>([]);
  const [pictures, setPictures] = useState<HelpPictures | null>(null);
  const [later, setLater] = useState(false);
  const [bringing, setBringing] = useState(false);
  const [trouble, setTrouble] = useState<string | null>(null);

  useEffect(() => {
    if (topic) setView({ kind: "markdown", topic });
  }, [topic]);

  useEffect(() => {
    let live = true;
    helpPages(spanish)
      .then(async (found) => {
        if (!live) return;
        setEntries(forTheWindow(found));
        const read = await Promise.all(
          found.map((one) => helpPage(spanish, one.slug).then((body) => [one.slug, body ?? ""])),
        );
        if (live) setBodies((was) => ({ ...was, ...Object.fromEntries(read) }));
      })
      .catch(() => undefined);
    helpPictures(spanish)
      .then((said) => live && setPictures(said))
      .catch(() => undefined);
    helpReferenceHere(spanish)
      .then((text) => {
        if (!live) return;
        const topics = topicsOf(text);
        const found = marksOf(topics, t("helpMarkdown"));
        setMarks(found);
        setBodies((was) => ({
          ...was,
          ...Object.fromEntries(found.map((one, at) => [one.slug, topics[at]?.body ?? ""])),
        }));
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [spanish]);

  if (view.kind === "markdown")
    return <MarkdownGuide topic={view.topic} onBack={() => setView({ kind: "home" })} />;

  const open = (slug: string, heading?: string, anchor?: string | null) => {
    if (slug.startsWith(MARKS))
      return setView({ kind: "markdown", topic: slug.slice(MARKS.length) });
    if (entries.some((one) => one.slug === slug)) setView({ kind: "page", slug, heading, anchor });
    else openLink(`${t("helpMoreAt")}${slug}/`).catch(() => undefined);
  };
  const markdown = () => setView({ kind: "markdown" });
  const bring = () => {
    setBringing(true);
    setTrouble(null);
    helpPicturesFetch(spanish)
      .then(setPictures)
      .catch((problem) => setTrouble(saidPlainly(problem)))
      .finally(() => setBringing(false));
  };
  const shown = view.kind === "page" ? entries.find((one) => one.slug === view.slug) : undefined;

  return (
    <main className="flex min-h-0 flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="flex min-h-0 flex-1">
        <nav
          aria-label={t("helpTopics")}
          className="scroller flex w-[230px] shrink-0 flex-col overflow-y-auto px-3 pb-6"
        >
          <button
            type="button"
            aria-current={view.kind === "home" ? "page" : undefined}
            onClick={() => setView({ kind: "home" })}
            className={item(view.kind === "home")}
          >
            {t("helpStart")}
          </button>
          {sectionsOf(entries).map(([section, pages]) => (
            <div key={section}>
              <p className={heading}>{section}</p>
              <ol className="flex flex-col gap-px">
                {pages.map((one) => (
                  <li key={one.slug}>
                    <button
                      type="button"
                      aria-current={shown?.slug === one.slug ? "page" : undefined}
                      onClick={() => open(one.slug)}
                      className={item(shown?.slug === one.slug)}
                    >
                      {one.title}
                    </button>
                  </li>
                ))}
              </ol>
            </div>
          ))}
          <p className={heading}>Markdown</p>
          <button type="button" onClick={markdown} className={item(false)}>
            {t("helpMarkdown")}
          </button>
          <button
            type="button"
            onClick={() => openLink(t("helpMoreAt")).catch(() => undefined)}
            className="mt-6 px-2.5 text-left text-[11.5px] text-faint hover:text-soft hover:underline"
          >
            {t("helpMore")} ↗
          </button>
        </nav>
        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          {pictures && pictures.missing > 0 && !later && (
            <div
              role="status"
              className="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-1.5 border-b border-hair px-6 py-2 text-[12.5px] text-soft"
            >
              <span className="min-w-0 flex-1">
                {fill("helpPictures", weigh(pictures.missing))}
              </span>
              <button
                type="button"
                disabled={bringing}
                onClick={bring}
                className={`rounded-md border border-line px-2.5 py-1 text-ink hover:bg-hover ${
                  bringing ? "motion-safe:animate-pulse" : ""
                }`}
              >
                {t(bringing ? "helpPicturesBringing" : "helpPicturesDo")}
              </button>
              {!bringing && (
                <button
                  type="button"
                  onClick={() => setLater(true)}
                  className="rounded-md px-2 py-1 text-faint hover:text-ink"
                >
                  {t("helpPicturesLater")}
                </button>
              )}
              {trouble && <span className="basis-full text-urgent">{trouble}</span>}
            </div>
          )}
          {view.kind === "home" && (
            <HelpHome
              entries={entries}
              more={marks}
              bodies={bodies}
              from={from}
              spanish={spanish}
              onOpen={open}
              onMarkdown={markdown}
            />
          )}
          {shown && view.kind === "page" && (
            <HelpArticle
              entry={shown}
              body={bodies[shown.slug] ?? ""}
              pictures={pictures}
              heading={view.heading}
              anchor={view.anchor ?? null}
              onOpen={open}
            />
          )}
        </div>
      </div>
    </main>
  );
}
