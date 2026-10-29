import { describe, expect, it, vi } from "vitest";
import { landscapeIn, paperIn, sheetsIn, withLandscape, withPaper, withSheets } from "../front";
import { seenIn } from "../ui/printing";

describe("the paper a document asks for", () => {
  it("reads it from the front matter, and A4 when it says nothing", () => {
    expect(paperIn("---\npaper: Letter\n---\n# Plan\n")).toBe("letter");
    expect(paperIn("---\npaper: legal\n---\n# Plan\n")).toBeNull();
    expect(paperIn("# Plan\n")).toBeNull();
  });

  it("writes another paper down and takes A4 away, keeping the rest as it was", () => {
    expect(withPaper("# Plan\n", "letter")).toBe("---\npaper: letter\n---\n# Plan\n");
    expect(withPaper("---\npaper: letter\n---\n# Plan\n", "a4")).toBe("# Plan\n");
    expect(withPaper("---\ntitle: Plan\npaper: letter\n---\nhola\n", "tabloid")).toBe(
      "---\ntitle: Plan\npaper: tabloid\n---\nhola\n",
    );
    expect(withPaper("---\ntitle: Plan\npaper: letter\n---\nhola\n", "a4")).toBe(
      "---\ntitle: Plan\n---\nhola\n",
    );
    expect(withPaper("# Plan\n", "a4")).toBe("# Plan\n");
    expect(withPaper("# Plan\r\nhola\r\n", "letter")).toBe(
      "---\r\npaper: letter\r\n---\r\n# Plan\r\nhola\r\n",
    );
  });

  it("keeps sheets off and upright unless the document asks, and writes only what it asks", () => {
    expect(sheetsIn("# Plan\n")).toBe(false);
    expect(landscapeIn("# Plan\n")).toBe(false);
    const asked = withLandscape(withSheets("# Plan\n", true), true);
    expect(asked).toBe("---\nsheets: on\norientation: landscape\n---\n# Plan\n");
    expect(sheetsIn(asked)).toBe(true);
    expect(landscapeIn(asked)).toBe(true);
    expect(withLandscape(withSheets(asked, false), false)).toBe("# Plan\n");
  });
});

describe("the page as it is seen, handed to the PDF", () => {
  it("says where the sheets turn and which folds are left open", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-5">uno</p><details open data-at="6-20"><summary>a</summary>x</details><details data-at="21-30"><summary>b</summary>y</details><div class="sheet-turn"></div><p data-at="31-40">dos</p></article>';

    expect(seenIn(document)).toEqual({ turns: ["31-40"], opened: ["6-20"], wide: [] });
    expect(seenIn(null)).toEqual({ turns: [], opened: [], wide: [] });
  });

  it("names the tables that, with every word whole, outgrow the paper's text", () => {
    document.body.innerHTML =
      '<article class="page"><div class="table" data-at="0-9"><table data-needs="900"></table></div><div class="table" data-at="10-19"><table data-needs="300"></table></div></article>';
    const offset = vi.spyOn(HTMLElement.prototype, "offsetWidth", "get");
    offset.mockImplementation(function (this: HTMLElement) {
      return this.style.width === "min-content" ? Number(this.dataset.needs ?? 0) : 640;
    });

    expect(seenIn(document, 643).wide).toEqual(["0-9"]);
    expect(seenIn(document).wide).toEqual([]);
    expect(document.querySelector<HTMLElement>("table")?.style.width).toBe("");
    offset.mockRestore();
  });
});
