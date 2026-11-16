import { afterEach, describe, expect, it } from "vitest";
import { checkOf } from "../placing";
import { ghostAt, sourceAt, spotOf, topsAt } from "../ui/pageSpots";

const BODY = "Uno :dog: **dos**\n\n- \n\nFin";
const PAGE =
  '<p data-at="0-17"><span data-src="0-4">Uno </span><span data-src="4-9" contenteditable="false">🐶</span><span data-src="9-10"> </span><strong><span data-src="12-15">dos</span></strong></p>' +
  '<ul data-at="19-21"><li data-at="19-21"></li></ul>' +
  '<p data-at="23-26"><span data-src="23-26">Fin</span></p>';

afterEach(() => {
  document.body.innerHTML = "";
});

const sheetOf = (html = PAGE) => {
  document.body.innerHTML = `<article class="page">${html}</article>`;
  return document.querySelector(".page") as HTMLElement;
};

const placed = (said: string) =>
  Array.from(document.querySelectorAll("[data-src]")).find(
    (one) => one.textContent === said,
  ) as Element;

describe("the places of a page written in place", () => {
  it("takes the end of a piece that cannot be written in, as an icon", () => {
    sheetOf();
    expect(sourceAt(BODY, placed("🐶"), 0)).toBe(9);
  });

  it("writes on the sheet's margin after the block before it, and at the start of an empty page", () => {
    const sheet = sheetOf();
    expect(sourceAt(BODY, sheet, 1)).toBe(17);
    expect(sourceAt(BODY, sheet, 0)).toBe(0);
    const empty = sheetOf("");
    expect(sourceAt("", empty, 0)).toBe(0);
  });

  it("lays the caret past a piece that cannot be written in, where the next words start", () => {
    sheetOf();
    const spot = spotOf(document, BODY, 9);
    expect(spot?.[0]).toBe(placed(" ").firstChild);
    expect(spot?.[1]).toBe(0);
  });

  it("lays the caret at the end of the words before a mark the page hides", () => {
    sheetOf();
    const spot = spotOf(document, BODY, 16);
    expect(spot?.[0]).toBe(placed("dos").firstChild);
    expect(spot?.[1]).toBe(3);
  });

  it("lays the caret inside an item with no words yet", () => {
    sheetOf();
    const item = document.querySelector("li") as Element;
    expect(spotOf(document, BODY, 21)).toEqual([item, 0]);
  });

  it("writes in a step just begun before the line break that ends it, not as a paragraph after", () => {
    const steps = "- [ ] un paso\n- [ ] \nFin";
    sheetOf(
      '<ul data-at="0-21"><li class="task" data-at="0-14"><span class="check"></span><span data-src="6-13">un paso</span></li>' +
        '<li class="task" data-at="14-21"><span class="check"></span></li></ul>' +
        '<p data-at="21-24"><span data-src="21-24">Fin</span></p>',
    );
    const empty = document.querySelectorAll("li")[1] as Element;
    expect(sourceAt(steps, empty, 1)).toBe(20);
  });

  it("finds no place outside every block", () => {
    sheetOf();
    expect(spotOf(document, BODY, 18)).toBeNull();
  });

  it("stands a paragraph not yet written after the block before it, or first on the page", () => {
    const sheet = sheetOf();
    const after = ghostAt(document, BODY, 18);
    expect(after?.previousElementSibling).toBe(sheet.firstElementChild);
    after?.remove();
    const first = ghostAt(document, BODY, 0);
    expect(sheet.firstElementChild).toBe(first);
  });

  it("begins the first line of an empty page where its title waits", () => {
    document.body.innerHTML = '<article class="page"><h1 data-title="Título"></h1></article>';
    const title = document.querySelector("h1");
    expect(ghostAt(document, "", 0)).toBe(title);
    expect(document.querySelectorAll(".page > *")).toHaveLength(1);
  });
});

describe("the blocks of a long page, found by halving", () => {
  const LONG = Array.from(
    { length: 41 },
    (_, at) =>
      `<p data-at="${at * 10}-${at * 10 + 8}"><span data-src="${at * 10}-${at * 10 + 8}">uno</span></p>`,
  ).join("");

  it("finds the block that holds a byte, past the stand-ins with no place", () => {
    const sheet = sheetOf(`<p data-ghost=""></p>${LONG}<h1 data-title=""></h1>`);
    for (const byte of [0, 7, 9, 205, 400, 408]) {
      const held = Array.from(sheet.querySelectorAll(":scope > [data-at]")).filter((one) => {
        const [from, to] = (one.getAttribute("data-at") ?? "").split("-").map(Number);
        return from <= byte && byte <= to;
      });
      expect(topsAt(sheet, byte)).toEqual(held);
    }
    expect(topsAt(sheet, 409)).toEqual([]);
  });

  it("writes on the margin of a long sheet after the block before the point", () => {
    const sheet = sheetOf(LONG);
    const text = "x".repeat(410);
    expect(sourceAt(text, sheet, 0)).toBe(0);
    expect(sourceAt(text, sheet, 21)).toBe(208);
    expect(sourceAt(text, sheet, 41)).toBe(408);
  });

  it("finds both blocks that meet at a byte", () => {
    const sheet = sheetOf('<p data-at="0-4"></p><p data-at="4-9"></p><p data-at="11-12"></p>');
    expect(topsAt(sheet, 4)).toEqual([sheet.children[0], sheet.children[1]]);
  });

  it("lays the caret and the stand-in alike counted from where the last key landed", () => {
    const text = "x".repeat(410);
    const known = { text, check: checkOf(text), unit: 205, byte: 205 };
    sheetOf(LONG);
    expect(spotOf(document, text, 203, known)).toEqual(spotOf(document, text, 203));
    const ghost = ghostAt(document, text, 209, known);
    expect(ghost?.previousElementSibling?.getAttribute("data-at")).toBe("200-208");
  });
});
