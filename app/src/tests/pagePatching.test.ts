import { afterEach, describe, expect, it } from "vitest";
import { patched } from "../ui/pagePatching";

const PAGE =
  '<p data-at="0-4"><span data-src="0-4">One.</span></p>' +
  '<p data-at="6-10"><span data-src="6-10">Two.</span></p>' +
  '<p data-at="12-18"><span data-src="12-18">Three.</span></p>';

afterEach(() => {
  document.body.innerHTML = "";
});

const sheetOf = () => {
  document.body.innerHTML = `<article class="page">${PAGE}</article>`;
  return document.querySelector(".page") as Element;
};

describe("a page written on, changed only where the core drew it again", () => {
  it("lays the blocks drawn again and moves those after by what was added", () => {
    const sheet = sheetOf();
    const last = sheet.lastElementChild;
    const laid = patched(
      sheet,
      {
        spans: [
          [0, 4],
          [6, 13],
          [15, 21],
        ],
        from: 1,
        to: 2,
        html: '<p data-at="6-13"><span data-src="6-13">Two too.</span></p>',
      },
      3,
    );
    expect(laid?.map((one) => one.textContent)).toEqual(["Two too."]);
    expect(Array.from(sheet.children, (one) => one.textContent)).toEqual([
      "One.",
      "Two too.",
      "Three.",
    ]);
    expect(sheet.lastElementChild).toBe(last);
    expect(last?.getAttribute("data-at")).toBe("15-21");
    expect(last?.querySelector("[data-src]")?.getAttribute("data-src")).toBe("15-21");
  });

  it("takes the place of every block a change swallows, as a fence left open", () => {
    const sheet = sheetOf();
    const laid = patched(
      sheet,
      {
        spans: [
          [0, 4],
          [6, 21],
        ],
        from: 1,
        to: 2,
        html: '<pre data-at="6-21">everything after</pre>',
      },
      3,
    );
    expect(laid).toHaveLength(1);
    expect(Array.from(sheet.children, (one) => one.textContent)).toEqual([
      "One.",
      "everything after",
    ]);
  });

  it("asks for the whole page when a block past the change is drawn otherwise", () => {
    const sheet = sheetOf();
    const laid = patched(
      sheet,
      {
        spans: [
          [0, 4],
          [6, 10],
          [12, 30],
        ],
        from: 1,
        to: 2,
        html: '<p data-at="6-10"><span data-src="6-10">Two.</span></p>',
      },
      0,
    );
    expect(laid).toBeNull();
    expect(sheet.children).toHaveLength(3);
  });

  it("asks for the whole page when a block it draws lends something of its own", () => {
    const sheet = sheetOf();
    const laid = patched(
      sheet,
      {
        spans: [
          [0, 4],
          [6, 10],
          [12, 18],
        ],
        from: 1,
        to: 2,
        html: '<figure class="widget" data-at="6-10"></figure>',
      },
      0,
    );
    expect(laid).toBeNull();
  });
});
