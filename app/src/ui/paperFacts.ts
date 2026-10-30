import type { DocFacts } from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { DOC } from "../markdown";
import { kept } from "../previews";

export const held = (pages: number): string =>
  pages === 1 ? t("pageHeld") : fill("pagesHeld", String(pages));

export const counted = (body: string, needle: string): number => body.split(needle).length - 1;

export const dated = (when: number | null): string =>
  when ? stamped(new Date(when * 1000).toISOString()) : "—";

// What a document holds besides its words, each said on its own: its pages, the documents and the files it names.
export const holdsOf = (body: string, facts: DocFacts | null): string[] => {
  const papers = counted(body, DOC);
  const files = [...body.matchAll(/\]\(<?([^)\s>]+)>?/g)].filter((one) => kept(one[1])).length;
  return [
    facts?.pages ? held(facts.pages) : "",
    papers > 0 ? fill("paperDocs", String(papers)) : "",
    files > 0 ? fill("paperFiles", String(files)) : "",
  ].filter(Boolean);
};
