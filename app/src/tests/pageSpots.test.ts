import { afterEach, describe, expect, it } from "vitest";
import { ghostAt, sourceAt, spotOf } from "../ui/pageSpots";

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
});
