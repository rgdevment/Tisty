import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
// @ts-expect-error the build's own script carries no types
import * as site from "../../scripts/guide-site.mjs";
import { drawAll } from "../guide/diagramming";
import { topicsOf } from "../guide/reference";

const {
  commitsIn,
  datesOf,
  describedAs,
  headers,
  LANGUAGES,
  mapped,
  raised,
  searchOnReach,
  sheetOf,
} = site;

// A mutation run works on a copy of the window alone, deep under it, so the reference is found by walking up.
const referenceFile = (file: string) => {
  let at = process.cwd();
  while (!existsSync(join(at, "crates", "tisty-markdown"))) {
    const up = dirname(at);
    if (up === at) throw new Error("no crates folder above the tests");
    at = up;
  }
  return join(at, "crates", "tisty-markdown", "reference", file);
};

vi.mock("tisty-editor/read", () => ({
  drawing: async (source: string, dark: boolean) => `${dark ? "dark" : "light"} ${source}`,
}));

describe("the guide's pages", () => {
  it("wear the page's own CSS, tables included, as the reader hands it", () => {
    const guide = readFileSync("scripts/guide.mjs", "utf8");
    expect(guide).toMatch(/PAGE_CSS: sheet/);
    expect(guide).not.toMatch(/page\.css\?raw/);
  });
});

describe("the guide's diagrams", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("are drawn in both looks, in fonts that cover their words, and handed back", async () => {
    const posted: { at: string; body?: string }[] = [];
    vi.stubGlobal("fetch", async (at: string, asked?: { body: string }) => {
      posted.push({ at, body: asked?.body });
      return { json: async () => ["one", "ñandú"] };
    });
    const load = vi.fn(async () => []);
    vi.stubGlobal("document", { fonts: { load } });

    await drawAll();

    expect(load).toHaveBeenCalledWith('16px "Inter Variable"', "oneñandú");
    expect(posted.map((one) => one.at)).toEqual(["/sources", "/drawn"]);
    expect(JSON.parse(posted[1].body ?? "")).toEqual([
      { source: "one", light: "light one", dark: "dark one" },
      { source: "ñandú", light: "light ñandú", dark: "dark ñandú" },
    ]);
  });
});

describe("the guide's search", () => {
  class Drawn {
    constructor({ element }: { element: Element }) {
      element.insertAdjacentHTML("beforeend", '<input class="found">');
    }
  }
  const nextTurn = () => new Promise((done) => setTimeout(done));

  const reachable = () => {
    document.head.innerHTML = "";
    document.body.innerHTML = '<div data-search><input class="search-waiting"></div>';
    searchOnReach();
    return document.querySelector("[data-search]") as HTMLElement;
  };

  const loading = () => ({
    sheet: document.head.querySelector("link") as HTMLLinkElement,
    code: document.head.querySelector("script") as HTMLScriptElement,
  });

  const landed = async () => {
    const { sheet, code } = loading();
    sheet.dispatchEvent(new Event("load"));
    code.dispatchEvent(new Event("load"));
    await nextTurn();
  };

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("downloads nothing until a reader reaches for the box, and then once", () => {
    const box = reachable();
    expect(document.head.querySelector("link, script")).toBeNull();

    box.dispatchEvent(new Event("pointerenter"));
    box.dispatchEvent(new Event("focusin"));

    expect(document.head.querySelectorAll("link")).toHaveLength(1);
    expect(document.head.querySelectorAll("script")).toHaveLength(1);
    expect(loading().sheet.getAttribute("href")).toBe("/pagefind/pagefind-ui.css");
    expect(loading().code.getAttribute("src")).toBe("/pagefind/pagefind-ui.js");
  });

  it("hands what was typed to the real box, and the focus with it", async () => {
    vi.stubGlobal("PagefindUI", Drawn);
    const box = reachable();
    const waiting = box.querySelector("input") as HTMLInputElement;
    waiting.focus();
    box.dispatchEvent(new Event("focusin"));
    waiting.value = "tabl";

    const typed: string[] = [];
    box.addEventListener("input", (event) => typed.push((event.target as HTMLInputElement).value));
    await landed();

    const field = box.querySelector("input") as HTMLInputElement;
    expect(field.className).toBe("found");
    expect(field.value).toBe("tabl");
    expect(typed).toEqual(["tabl"]);
    expect(document.activeElement).toBe(field);
  });

  it("leaves the focus alone when the reader only passed over the box", async () => {
    vi.stubGlobal("PagefindUI", Drawn);
    const box = reachable();
    box.dispatchEvent(new Event("pointerenter"));
    await landed();

    expect(box.querySelectorAll("input")).toHaveLength(1);
    expect(document.activeElement).toBe(document.body);
  });

  it("steps aside when a file does not arrive", async () => {
    const box = reachable();
    box.dispatchEvent(new Event("pointerenter"));

    loading().sheet.dispatchEvent(new Event("load"));
    loading().code.dispatchEvent(new Event("error"));
    await nextTurn();

    expect(box.isConnected).toBe(false);
  });

  it("steps aside when Pagefind cannot draw its box", async () => {
    vi.stubGlobal(
      "PagefindUI",
      class {
        constructor() {
          throw new Error("no");
        }
      },
    );
    const box = reachable();
    box.dispatchEvent(new Event("pointerenter"));
    await landed();

    expect(box.isConnected).toBe(false);
  });

  it("steps aside when Pagefind draws no box", async () => {
    vi.stubGlobal("PagefindUI", class {});
    const box = reachable();
    box.dispatchEvent(new Event("pointerenter"));
    await landed();

    expect(box.isConnected).toBe(false);
  });
});

describe("what the guide tells a search engine", () => {
  type Language = { code: string; file: string; image: string; said: Record<string, string> };
  const languages: Language[] = LANGUAGES;

  const page = (slug: string, title: string) => ({
    at: slug === "index" ? 0 : 1,
    slug,
    title,
    label: title,
    subs: [],
    said: "What it says.",
    published: "2026-04-01",
    changed: "2026-05-02",
    html: "<article></article>",
  });
  const built = new Map(
    languages.map((language) => [language.code, [page("index", "Home"), page("tables", "Tables")]]),
  );
  const look = { style: "", logo: "logo.svg", font: "/fonts/inter.woff2" };
  const dataIn = (html: string) =>
    [...html.matchAll(/<script type="application\/ld\+json">(.*?)<\/script>/g)].map((one) =>
      JSON.parse(one[1]),
    );

  it.each(languages.map((one) => [one.code, one] as const))(
    "describes each topic in %s, in what a result page shows whole",
    (_, language) => {
      const text = readFileSync(referenceFile(language.file), "utf8");
      const slugs = topicsOf(text).map((one) => one.slug);

      expect(Object.keys(language.said).sort()).toEqual([...slugs].sort());
      for (const said of Object.values(language.said)) {
        expect(said.length).toBeGreaterThan(70);
        expect(said.length).toBeLessThanOrEqual(160);
      }
    },
  );

  it("says what to do when a topic has no description", () => {
    const [english] = languages;

    expect(describedAs(english, "tables")).toBe(english.said.tables);
    expect(() => describedAs(english, "brand-new")).toThrow(/no description for "brand-new"/);
  });

  it("names the start page for what it is, and the site by its name", () => {
    const [english] = languages;
    const html: string = sheetOf(built, english, built.get(english.code)?.[0], look);

    expect(html).toContain("<title>Tisty Markdown guide: syntax and examples</title>");
    const [site, ...others] = dataIn(html);
    expect(site["@type"]).toBe("WebSite");
    expect(site.name).toBe("Tisty Markdown");
    expect(others).toEqual([]);
  });

  it("dates a topic and places it under the start page", () => {
    const [, spanish] = languages;
    const html: string = sheetOf(built, spanish, built.get(spanish.code)?.[1], look);

    const [article, trail] = dataIn(html);
    expect(article["@type"]).toBe("TechArticle");
    expect(article.datePublished).toBe("2026-04-01");
    expect(article.dateModified).toBe("2026-05-02");
    expect(article.author.name).toBe("Tisty");
    expect(article.image).toBe(spanish.image);
    expect(trail["@type"]).toBe("BreadcrumbList");
    expect(trail.itemListElement).toEqual([
      {
        "@type": "ListItem",
        position: 1,
        name: "Markdown de Tisty",
        item: "https://markdown.tisty.app/es/",
      },
      {
        "@type": "ListItem",
        position: 2,
        name: "Tables",
        item: "https://markdown.tisty.app/es/tables",
      },
    ]);
  });

  it("dates every address in the sitemap", () => {
    const map: string = mapped(built);

    expect(map).toContain(
      "<loc>https://markdown.tisty.app/tables</loc><lastmod>2026-05-02</lastmod>",
    );
    expect(map.match(/<lastmod>/g)).toHaveLength(4);
  });

  it("leaves a page undated when the history cannot say, rather than guess", () => {
    const { changed: _, published: __, ...undated } = page("tables", "Tables");
    const none = new Map(languages.map((language) => [language.code, [undated]]));

    expect(mapped(none)).not.toContain("lastmod");
    const [article] = dataIn(sheetOf(none, languages[0], undated, look));
    expect(article.dateModified).toBeUndefined();
    expect(article.datePublished).toBeUndefined();
  });

  it("reads the commits of a log, skipping an empty one", () => {
    expect(commitsIn("abc 2026-11-03\ndef 2026-10-01\n")).toEqual([
      { hash: "abc", date: "2026-11-03" },
      { hash: "def", date: "2026-10-01" },
    ]);
    expect(commitsIn("")).toEqual([]);
  });

  it("gives a topic's page one top heading, moving its own up a level", () => {
    const page = new DOMParser().parseFromString(
      '<article><h2 id="tables" class="a">Tables <em>now</em></h2><p>Words</p><h3 id="types">Types</h3><h4>Foot</h4><figure><h2>Example</h2></figure></article>',
      "text/html",
    );

    raised(page);

    const heads = [...page.querySelectorAll("article > *")].map((one) => one.localName);
    expect(heads).toEqual(["h1", "p", "h2", "h3", "figure"]);
    const top = page.querySelector("h1");
    expect(top?.id).toBe("tables");
    expect(top?.className).toBe("a");
    expect(top?.innerHTML).toBe("Tables <em>now</em>");
    expect(page.querySelector("figure > h2")?.textContent).toBe("Example");
  });

  it("keeps an example's headings out of the page's outline, on every kind of page", () => {
    const [topic, start] = [
      '<article><h2>Title</h2><figure class="example"><div class="drawn"><h1>Sample</h1><h3>Part</h3></div></figure></article>',
      '<article><h1>Guide</h1><figure class="example"><div class="drawn"><h2>Sample</h2></div></figure></article>',
    ].map((html) => new DOMParser().parseFromString(html, "text/html"));

    raised(topic);
    raised(start);

    expect(topic.querySelector("article > h1")?.getAttribute("role")).toBeNull();
    expect(topic.querySelector(".drawn h1")?.getAttribute("role")).toBe("presentation");
    expect(topic.querySelector(".drawn h3")?.getAttribute("role")).toBe("presentation");
    expect(start.querySelector("article > h1")?.getAttribute("role")).toBeNull();
    expect(start.querySelector(".drawn h2")?.getAttribute("role")).toBe("presentation");
  });

  it("leaves a page that already has its top heading", () => {
    const page = new DOMParser().parseFromString(
      "<article><h1>Guide</h1><h2>Glance</h2></article>",
      "text/html",
    );

    raised(page);

    expect([...page.querySelectorAll("article > *")].map((one) => one.localName)).toEqual([
      "h1",
      "h2",
    ]);
  });

  it("dates a topic by when it last changed, not by the last release", () => {
    const topics = (tables: string, tags?: string) => [
      { slug: "tables", body: tables },
      ...(tags === undefined ? [] : [{ slug: "tags", body: tags }]),
    ];

    const dates = datesOf([
      { date: "2026-11-03", topics: topics("new", "same") },
      { date: "2026-10-01", topics: topics("old", "same") },
      { date: "2026-09-01", topics: topics("new", "same") },
      { date: "2026-08-01", topics: topics("old") },
      { date: "2026-07-01", topics: topics("old", "same") },
    ]);

    expect(dates.get("tables")).toEqual({ published: "2026-07-01", changed: "2026-11-03" });
    expect(dates.get("tags")).toEqual({ published: "2026-07-01", changed: "2026-09-01" });
  });
});

describe("the guide's cache", () => {
  const pages = (many: number) =>
    new Map([
      [
        "en",
        [
          { slug: "index" },
          ...Array.from({ length: many - 1 }, (_, at) => ({ slug: `topic-${at}` })),
        ],
      ],
      ["es", [{ slug: "index" }]],
    ]);

  it("names each page, so no rule joins its values with another's", () => {
    const rules: string = headers("logo.svg", pages(2));

    expect(rules).toContain("/\n  Cache-Control: public, max-age=300, stale-while-revalidate=3600");
    expect(rules).toContain(
      "/topic-0\n  Cache-Control: public, max-age=300, stale-while-revalidate=3600",
    );
    expect(rules).toContain(
      "/es/\n  Cache-Control: public, max-age=300, stale-while-revalidate=3600",
    );
    expect(rules).toContain("/logo.svg\n  Cache-Control: public, max-age=31536000, immutable");
    expect(rules).toContain("/fonts/*\n  Cache-Control: public, max-age=31536000, immutable");
    expect(rules).toContain("/es/rina.jpg\n  Cache-Control: public, max-age=86400");
    expect(rules.split("\n")).not.toContain("/*");
    expect(rules.endsWith("\n")).toBe(true);
  });

  it("refuses more rules than Pages applies, instead of losing the last pages' cache", () => {
    expect(() => headers("logo.svg", pages(93))).not.toThrow();
    expect(() => headers("logo.svg", pages(94))).toThrow(/101 header rules/);
  });
});
