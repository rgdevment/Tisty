import { useEffect, useRef, useState } from "react";
import rina from "../../src-tauri/resources/guide/en/rina.jpg";
import { helpAnchors, helpDrawn, helpReference, openLink } from "../core";
import { examplesIn, shownBeside, type Topic, topicsOf } from "../guide/reference";
import { locale, t } from "../locales";
import { pictured } from "../previews";
import type { Carded } from "./carding";
import Page from "./Page";
import type { Head } from "./paging";

const titles = new Map<string, string>();
const drawings = new Map<string, Promise<string>>();

// The examples name files only a store holds, so a picture shows the guide's own photo.
const drawn = async (body: string): Promise<string> => {
  const page = new DOMParser().parseFromString(
    `<article>${await helpDrawn(body)}</article>`,
    "text/html",
  );
  const examples = examplesIn(page);
  const looks = await Promise.all(examples.map((code) => helpDrawn(code.textContent ?? "")));
  const words = { written: t("helpWritten"), drawn: t("helpDrawn") };
  examples.forEach((code, at) => {
    shownBeside(code, looks[at], words);
  });
  const photo = new URL(rina, location.href).href;
  for (const img of page.querySelectorAll("img[src]")) {
    const at = img.getAttribute("src") ?? "";
    if (!/^[a-z][a-z0-9+.-]*:/i.test(at) && pictured(at)) img.setAttribute("src", photo);
  }
  for (const card of page.querySelectorAll<HTMLElement>("a.card[data-doc]"))
    titles.set(card.dataset.doc ?? "", card.textContent ?? "");
  return page.querySelector("article")?.innerHTML ?? "";
};

const drawGuide = (body: string): Promise<string> => {
  const known = drawings.get(body);
  if (known) return known;
  const made = drawn(body);
  drawings.set(body, made);
  made.catch(() => drawings.delete(body));
  return made;
};

const card = (file: string): Carded => ({ kind: "doc", title: titles.get(file) ?? "" });

const pill =
  "inline-flex max-w-[48%] items-center gap-2 rounded-full border border-line bg-sheet px-4 py-2 text-[13px] font-medium text-ink hover:border-accent hover:text-accent";

export default function Help({ topic }: { readonly topic?: string }) {
  const [topics, setTopics] = useState<Topic[] | null>(null);
  const [anchors, setAnchors] = useState<string[][]>([]);
  const [failed, setFailed] = useState(false);
  const [slug, setSlug] = useState(topic ?? "index");
  const [heads, setHeads] = useState<Head[]>([]);
  const aimed = useRef<string | null>(null);
  const view = useRef<Window | null>(null);

  useEffect(() => {
    helpReference(locale().toLowerCase().startsWith("es"))
      .then((text) => {
        const found = topicsOf(text);
        setTopics(found);
        return helpAnchors(found.map((one) => one.body)).then(setAnchors);
      })
      .catch(() => setFailed(true));
  }, []);

  useEffect(() => {
    if (topic) setSlug(topic);
  }, [topic]);

  const at = Math.max(0, topics?.findIndex((one) => one.slug === slug) ?? 0);
  const shown = topics?.[at];
  const named = (one: Topic) => one.heading ?? t("helpStart");
  const open = (next: string, anchor: string | null = null) => {
    aimed.current = anchor;
    setHeads([]);
    setSlug(next);
  };

  const onHere = (id: string) => {
    const owner = topics?.findIndex((_, one) => anchors[one]?.includes(id)) ?? -1;
    if (owner < 0 || !topics) return;
    if (owner === at) {
      view.current?.document.getElementById(id)?.scrollIntoView({ block: "start" });
      return;
    }
    open(topics[owner].slug, anchors[owner][0] === id ? null : id);
  };

  const before = topics?.[at - 1];
  const after = topics?.[at + 1];
  const subs = heads.filter((one) => one.level === 3);

  return (
    <main className="flex min-h-0 flex-col overflow-hidden">
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="flex min-h-0 flex-1">
        <nav
          aria-label={t("helpTopics")}
          className="scroller flex w-[230px] shrink-0 flex-col overflow-y-auto px-3 pb-6"
        >
          <p className="px-2.5 pb-2 text-[11.5px] font-semibold tracking-wide text-faint uppercase">
            {t("help")}
          </p>
          <ol className="flex flex-col gap-px">
            {topics?.map((one) => {
              const here = one.slug === shown?.slug;
              return (
                <li key={one.slug}>
                  <button
                    type="button"
                    aria-current={here ? "page" : undefined}
                    onClick={() => open(one.slug)}
                    className={`w-full rounded-md px-2.5 py-1.5 text-left text-[13px] leading-snug ${
                      here
                        ? "bg-accent-soft font-semibold text-accent"
                        : "text-soft hover:bg-hover hover:text-ink"
                    }`}
                  >
                    {named(one)}
                  </button>
                  {here && subs.length > 0 && (
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
          <button
            type="button"
            onClick={() => openLink(t("helpMoreAt")).catch(() => undefined)}
            className="mt-5 px-2.5 text-left text-[12.5px] text-accent hover:underline"
          >
            {t("helpMore")} ↗
          </button>
        </nav>
        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          {failed && <p className="px-6 pt-6 text-[12.5px] text-urgent">{t("helpGone")}</p>}
          {shown && (
            <Page
              key={shown.slug}
              body={shown.body}
              draw={drawGuide}
              label={named(shown)}
              stranger={false}
              card={card}
              onDoc={() => undefined}
              onHere={onHere}
              onOutline={setHeads}
              onShown={(seen) => {
                view.current = seen;
                if (!seen || !aimed.current) return;
                seen.document.getElementById(aimed.current)?.scrollIntoView({ block: "start" });
                aimed.current = null;
              }}
            />
          )}
          {shown && (
            <nav
              aria-label={`${t("helpBefore")} · ${t("helpAfter")}`}
              className="flex shrink-0 gap-3 border-t border-hair px-6 py-3"
            >
              {before && (
                <button
                  type="button"
                  title={t("helpBefore")}
                  className={pill}
                  onClick={() => open(before.slug)}
                >
                  <span aria-hidden="true" className="text-faint">
                    ←
                  </span>
                  <span className="truncate">{named(before)}</span>
                </button>
              )}
              {after && (
                <button
                  type="button"
                  title={t("helpAfter")}
                  className={`${pill} ml-auto`}
                  onClick={() => open(after.slug)}
                >
                  <span className="truncate">{named(after)}</span>
                  <span aria-hidden="true" className="text-faint">
                    →
                  </span>
                </button>
              )}
            </nav>
          )}
        </div>
      </div>
    </main>
  );
}
