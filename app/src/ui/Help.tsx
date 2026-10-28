import { useEffect, useMemo, useRef, useState } from "react";
import { openLink } from "../core";
import { weigh } from "../format";
import {
  type HelpEntry,
  type HelpPictures,
  helpPage,
  helpPages,
  helpPictures,
  helpPicturesFetch,
  helpReference,
  helpReferenceHere,
} from "../guide/calls";
import { forTheWindow, guideAt, MARKS, marksOf, outlined, sectionsOf } from "../guide/help";
import { type Topic, topicsOf } from "../guide/reference";
import { fill, locale, t } from "../locales";
import { saidPlainly } from "../refusal";
import HelpArticle from "./HelpArticle";
import HelpHome from "./HelpHome";
import HelpNav from "./HelpNav";
import HelpSteps from "./HelpSteps";
import MarkdownArticle from "./MarkdownArticle";
import type { Head } from "./paging";

type View =
  | { kind: "home" }
  | { kind: "page"; slug: string; heading?: string; anchor?: string | null };

const viewOf = (topic?: string): View =>
  topic ? { kind: "page", slug: `${MARKS}${topic}` } : { kind: "home" };

export default function Help({ topic, from }: { readonly topic?: string; readonly from?: string }) {
  const spanish = locale().toLowerCase().startsWith("es");
  const [view, setView] = useState<View>(viewOf(topic));
  const [entries, setEntries] = useState<HelpEntry[]>([]);
  const [bodies, setBodies] = useState<Record<string, string>>({});
  const [topics, setTopics] = useState<Topic[]>([]);
  const [heads, setHeads] = useState<Head[]>([]);
  const [pictures, setPictures] = useState<HelpPictures | null>(null);
  const [later, setLater] = useState(false);
  const [bringing, setBringing] = useState(false);
  const [trouble, setTrouble] = useState<string | null>(null);
  const [gone, setGone] = useState(false);
  const fresh = useRef(false);

  useEffect(() => {
    if (topic) setView(viewOf(topic));
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
        if (live) setBodies(Object.fromEntries(read));
      })
      .catch(() => undefined);
    helpPictures(spanish)
      .then((said) => live && setPictures(said))
      .catch(() => undefined);
    helpReferenceHere(spanish)
      .then((text) => live && setTopics((was) => (was.length > 0 ? was : topicsOf(text))))
      .catch(() => live && setGone(true));
    return () => {
      live = false;
    };
  }, [spanish]);

  const reading = view.kind === "page" && view.slug.startsWith(MARKS);
  useEffect(() => {
    if (!reading || fresh.current) return;
    fresh.current = true;
    helpReference(spanish)
      .then((text) => setTopics(topicsOf(text)))
      .catch(() => undefined);
  }, [reading, spanish]);

  const marks = useMemo(() => marksOf(topics, t("helpMarkdown")), [topics]);
  const sections = sectionsOf(entries);
  const searchable = useMemo(
    () => ({
      ...bodies,
      ...Object.fromEntries(marks.map((one, at) => [one.slug, topics[at]?.body ?? ""])),
    }),
    [bodies, marks, topics],
  );

  // A page the window does not carry, or a guide it could not read, opens on the site instead.
  const open = (slug: string, heading?: string, anchor?: string | null) => {
    const guide = slug.startsWith(MARKS);
    const here = guide
      ? !gone && (topics.length === 0 || marks.some((one) => one.slug === slug))
      : entries.some((one) => one.slug === slug);
    if (here) {
      setHeads([]);
      setView({ kind: "page", slug, heading, anchor });
      return;
    }
    const away = guide
      ? guideAt(slug.slice(MARKS.length), anchor, spanish)
      : `${t("helpMoreAt")}${slug}/${anchor ? `#${anchor}` : ""}`;
    openLink(away).catch(() => undefined);
  };
  const home = () => {
    setHeads([]);
    setView({ kind: "home" });
  };
  const bring = () => {
    setBringing(true);
    setTrouble(null);
    helpPicturesFetch(spanish)
      .then((said) => {
        setPictures(said);
        if (said.missing > 0) setTrouble(t("helpPicturesSome"));
      })
      .catch((problem) => setTrouble(saidPlainly(problem)))
      .finally(() => setBringing(false));
  };

  const at = reading
    ? Math.max(
        0,
        marks.findIndex((one) => view.kind === "page" && one.slug === view.slug),
      )
    : -1;
  const shown =
    view.kind !== "page"
      ? undefined
      : reading
        ? marks[at]
        : entries.find((one) => one.slug === view.slug);
  const walked = [...sections.flatMap(([, pages]) => pages), ...marks];
  const step = shown ? walked.findIndex((one) => one.slug === shown.slug) : -1;

  return (
    <main className="flex min-h-0 flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="flex min-h-0 flex-1">
        <HelpNav
          sections={sections}
          marks={marks}
          here={shown?.slug ?? null}
          subs={outlined(heads)}
          onHome={home}
          onOpen={(slug) => open(slug)}
        />
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
              bodies={searchable}
              from={from}
              spanish={spanish}
              onOpen={open}
              onMarkdown={() => open(`${MARKS}index`)}
            />
          )}
          {shown && view.kind === "page" && !reading && (
            <HelpArticle
              entry={shown}
              body={bodies[shown.slug] ?? ""}
              pictures={pictures}
              heading={view.heading}
              anchor={view.anchor ?? null}
              onOpen={open}
              onOutline={setHeads}
              spanish={spanish}
            />
          )}
          {reading && !shown && gone && (
            <p className="px-6 pt-6 text-[12.5px] text-urgent">{t("helpGone")}</p>
          )}
          {shown && view.kind === "page" && reading && topics[at] && (
            <MarkdownArticle
              topic={topics[at]}
              topics={topics}
              anchor={view.anchor ?? null}
              onOpen={(slug, anchor) => open(`${MARKS}${slug}`, undefined, anchor)}
              onHelp={(slug, anchor) => open(slug, undefined, anchor)}
              spanish={spanish}
              onOutline={setHeads}
            />
          )}
          {shown && (
            <HelpSteps
              before={walked[step - 1]}
              after={step >= 0 ? walked[step + 1] : undefined}
              onOpen={(slug) => open(slug)}
            />
          )}
        </div>
      </div>
    </main>
  );
}
