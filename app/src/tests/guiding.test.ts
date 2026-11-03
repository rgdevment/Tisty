import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
// @ts-expect-error the build's own script carries no types
import { headers, searchOnReach } from "../../scripts/guide-site.mjs";
import { drawAll } from "../guide/diagramming";

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
