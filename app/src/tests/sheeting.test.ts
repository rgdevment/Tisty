import { laidIn } from "tisty-editor/read";
import { describe, expect, it, vi } from "vitest";
import { landscapeIn, sheetsIn, withDesign, withLandscape, withPaper, withSheets } from "../front";
import { seenIn, seenNow } from "../ui/printing";

describe("the paper a document asks for", () => {
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

describe("the design a document asks for", () => {
  it("is written by its name, and Tisty leaves nothing written", () => {
    const asked = withDesign(withPaper("# Plan\n", "letter"), "coral");
    expect(asked).toBe("---\npaper: letter\ndesign: coral\n---\n# Plan\n");
    expect(laidIn(asked).design).toBe("coral");
    expect(withDesign(asked, "tisty")).toBe("---\npaper: letter\n---\n# Plan\n");
  });

  it("is no business of a line of the same name inside another property", () => {
    const nested =
      "---\nproyecto:\n  design: Ana\n  code: Luis\nnotas: |\n  design: azul\n---\n# Plan\n";
    const coral = withDesign(nested, "coral");
    expect(coral).toBe(nested.replace("\n---\n# Plan", "\ndesign: coral\n---\n# Plan"));
    expect(withDesign(coral, "tisty")).toBe(nested);
    expect(withDesign(nested, "tisty")).toBe(nested);
  });

  it("takes its own indented values along when it goes", () => {
    const listed = "---\ntitle: Plan\ndesign:\n  - coral\npaper: letter\n---\n# Plan\n";
    expect(withDesign(listed, "tisty")).toBe("---\ntitle: Plan\npaper: letter\n---\n# Plan\n");
  });

  it("stays written when it is the person's own and Default is asked for again", () => {
    const theirs = "---\ndesign: https://figma.com/x\n---\n# Plan\n";
    expect(withDesign(theirs, "tisty")).toBe(theirs);
  });

  it("stays written when nobody knows it, and the page is Tisty", () => {
    const unknown = "---\ndesign: pergamino\n---\n# Plan\n";
    expect(laidIn(unknown).design).toBe("tisty");
    expect(withPaper(unknown, "letter")).toContain("design: pergamino");
  });
});

describe("the page as it is seen, handed to the PDF", () => {
  it("reads a page drawn anew from the words now while the whole document is being written", async () => {
    document.body.innerHTML =
      '<article class="page whole"><details open data-at="0-5"><summary>old</summary></details><div class="editing"></div></article>';
    const draw = vi.fn(() =>
      Promise.resolve('<details open data-at="7-12"><summary>new</summary></details>'),
    );
    expect(await seenNow(document, "# now", draw)).toEqual({
      turns: [],
      opened: ["7-12"],
      wide: [],
    });
    expect(draw).toHaveBeenCalledWith("# now");
    expect(document.querySelectorAll(".page")).toHaveLength(1);
  });

  it("reads the page on screen when nothing is being written whole", async () => {
    document.body.innerHTML =
      '<article class="page"><details open data-at="0-5"></details></article>';
    const draw = vi.fn(() => Promise.resolve(""));
    expect((await seenNow(document, "x", draw)).opened).toEqual(["0-5"]);
    expect(draw).not.toHaveBeenCalled();
  });

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
