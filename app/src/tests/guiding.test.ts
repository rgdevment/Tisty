import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
// @ts-expect-error the build's own script carries no types
import { headers } from "../../scripts/guide-site.mjs";
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
