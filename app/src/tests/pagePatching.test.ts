import { afterEach, describe, expect, it } from "vitest";
import { type PlacedNear, summed } from "../placing";
import { patched } from "../ui/pagePatching";

type Span = readonly [number, number];

const spansSum = (spans: readonly Span[]) => summed(spans.flat());

// The core answers how many blocks there are and a sum of those it did not draw again.
const near = (drawn: {
  spans: readonly Span[];
  from: number;
  to: number;
  html: string;
}): PlacedNear => ({
  count: drawn.spans.length,
  sum: spansSum([...drawn.spans.slice(0, drawn.from), ...drawn.spans.slice(drawn.to)]),
  from: drawn.from,
  to: drawn.to,
  html: drawn.html,
});

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
      near({
        spans: [
          [0, 4],
          [6, 13],
          [15, 21],
        ],
        from: 1,
        to: 2,
        html: '<p data-at="6-13"><span data-src="6-13">Two too.</span></p>',
      }),
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

  it("moves the place a card or a player keeps for its file with the block after", () => {
    const sheet = sheetOf();
    const last = sheet.lastElementChild as Element;
    last.innerHTML = '<a class="doc-card is-file" data-placed="12-18"></a>';
    patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 13],
          [15, 21],
        ],
        from: 1,
        to: 2,
        html: '<p data-at="6-13"><span data-src="6-13">Two too.</span></p>',
      }),
      3,
    );
    expect(last.querySelector("[data-placed]")?.getAttribute("data-placed")).toBe("15-21");
  });

  it("takes the place of every block a change swallows, as a fence left open", () => {
    const sheet = sheetOf();
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 21],
        ],
        from: 1,
        to: 2,
        html: '<pre data-at="6-21">everything after</pre>',
      }),
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
      near({
        spans: [
          [0, 4],
          [6, 10],
          [12, 30],
        ],
        from: 1,
        to: 2,
        html: '<p data-at="6-10"><span data-src="6-10">Two.</span></p>',
      }),
      0,
    );
    expect(laid).toBeNull();
    expect(sheet.children).toHaveLength(3);
  });

  it("asks for the whole page when a block before the change is drawn otherwise", () => {
    const sheet = sheetOf();
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 5],
          [7, 11],
          [13, 19],
        ],
        from: 1,
        to: 2,
        html: '<p data-at="7-11"><span data-src="7-11">Two.</span></p>',
      }),
      0,
    );
    expect(laid).toBeNull();
    expect(sheet.children).toHaveLength(3);
  });

  it("sums the places of blocks as the core does", () => {
    const three: Span[] = [
      [0, 4],
      [6, 13],
      [15, 21],
    ];
    expect(spansSum(three)).toEqual([0xc7956160, 0x9bfc67d1]);
    expect(spansSum([])).toEqual([0x811c9dc5, 0x9747b28c]);
  });

  it("asks for the whole page when a block it draws lends something of its own", () => {
    const sheet = sheetOf();
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 10],
          [12, 18],
        ],
        from: 1,
        to: 2,
        html: '<figure class="widget" data-at="6-10"></figure>',
      }),
      0,
    );
    expect(laid).toBeNull();
  });

  it("keeps a fold open while words are written in it, as it was before the key", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-4"><span data-src="0-4">One.</span></p>' +
      '<details data-at="6-30" open><summary>Fold</summary><p data-at="20-24"><span data-src="20-24">inne</span></p></details></article>';
    const sheet = document.querySelector(".page") as Element;
    patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 31],
        ],
        from: 1,
        to: 2,
        html: '<details data-at="6-31"><summary>Fold</summary><p data-at="20-25"><span data-src="20-25">inner</span></p></details>',
      }),
      1,
    );
    const fold = sheet.querySelector("details") as HTMLDetailsElement;
    expect(fold.textContent).toContain("inner");
    expect(fold.open).toBe(true);
  });
});
