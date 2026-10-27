import { describe, expect, it, vi } from "vitest";
import { landscapeIn, paperIn, sheetsIn, withLandscape, withPaper, withSheets } from "../front";
import { seenIn } from "../ui/printing";
import { sheeted } from "../ui/sheeting";

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

const laid = (blocks: [string, number, number][]) => {
  document.body.innerHTML = `<article class="page">${blocks.map(([tag]) => (tag === "hr" ? "<hr>" : `<${tag}></${tag}>`)).join("")}</article>`;
  const sheet = document.querySelector<HTMLElement>(".page") as HTMLElement;
  Object.defineProperty(sheet, "offsetTop", { value: 0 });
  // The page has no layout in the test window, so each block says where it sits and how tall it is.
  Array.from(sheet.children).forEach((one, at) => {
    Object.defineProperty(one, "offsetTop", { get: () => blocks[at][1] + turnsBefore(one) });
    Object.defineProperty(one, "offsetHeight", { value: blocks[at][2] });
  });
  return sheet;
};

const turnsBefore = (one: Element) => {
  let gap = 0;
  for (let at = one.previousElementSibling; at; at = at.previousElementSibling) {
    if (at.classList.contains("sheet-turn")) gap += 200;
  }
  return gap;
};

describe("sheets laid on the page", () => {
  it("starts a block that would cross the foot of a sheet on the next one", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["p", 60, 40],
      ["p", 110, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll(".sheet-turn");
    expect(turns).toHaveLength(1);
    expect(turns[0].nextElementSibling).toBe(sheet.children[3]);
    expect(sheet.style.getPropertyValue("--last")).not.toBe("");
  });

  it("leaves a block taller than a sheet where it starts, instead of a blank sheet before it", () => {
    const sheet = laid([
      ["table", 20, 500],
      ["p", 530, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll(".sheet-turn");
    expect(turns).toHaveLength(1);
    expect(turns[0].nextElementSibling?.tagName).toBe("P");
  });

  it("takes a heading left at the foot of a sheet to the next one, with what it heads", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["h2", 60, 30],
      ["h3", 90, 20],
      ["p", 110, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll<HTMLElement>(".sheet-turn");
    expect(turns).toHaveLength(1);
    expect(turns[0].nextElementSibling?.tagName).toBe("H2");
    expect(turns[0].style.getPropertyValue("--fill")).toBe("79px");
  });

  it("leaves a heading that already opens its sheet where it is", () => {
    const sheet = laid([
      ["p", 10, 120],
      ["h2", 130, 30],
      ["p", 160, 200],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll(".sheet-turn");
    expect(turns).toHaveLength(2);
    expect(turns[0].nextElementSibling?.tagName).toBe("H2");
    expect(turns[1].nextElementSibling?.tagName).toBe("P");
  });

  it("lets a manual break end its sheet and lays nothing while a block is being written", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["hr", 60, 50],
      ["p", 120, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    expect(sheet.querySelectorAll(".sheet-turn")).toHaveLength(0);
    expect((sheet.children[1] as HTMLElement).style.getPropertyValue("--fill")).toBe("79px");
    sheet.insertAdjacentHTML("beforeend", '<div class="editing"></div>');
    (sheet.children[1] as HTMLElement).style.removeProperty("--fill");
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    expect((sheet.children[1] as HTMLElement).style.getPropertyValue("--fill")).toBe("");
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
