import { afterEach, describe, expect, it } from "vitest";
import { namesSum, type PlacedNear, summed } from "../placing";
import { patched } from "../ui/pagePatching";
import { placeOf, placesSettled } from "../ui/pagePlaces";

type Span = readonly [number, number];

const spansSum = (spans: readonly Span[]) => summed(spans.flat());

// The core answers how many blocks there are and a sum of those it did not draw again.
const near = (drawn: {
  spans: readonly Span[];
  from: number;
  to: number;
  html: string;
  heads?: readonly string[];
  whole?: boolean;
}): PlacedNear => ({
  count: drawn.spans.length,
  sum: spansSum([...drawn.spans.slice(0, drawn.from), ...drawn.spans.slice(drawn.to)]),
  from: drawn.from,
  to: drawn.to,
  html: drawn.html,
  heads: drawn.heads ? namesSum(drawn.heads) : null,
  whole: drawn.whole ?? false,
});

const HEADED =
  '<h1 id="uno" data-at="0-5"><span data-src="2-5">Uno</span></h1>' +
  '<p data-at="7-9"><span data-src="7-9">A.</span></p>' +
  '<p data-at="11-13"><span data-src="11-13">B.</span></p>' +
  '<h1 id="uno-1" data-at="15-20"><span data-src="17-20">Uno</span></h1>';

const headedSheet = () => {
  document.body.innerHTML = `<article class="page">${HEADED}</article>`;
  return document.querySelector(".page") as Element;
};

describe("headings a page keeps, named by those before them", () => {
  it("asks for the whole page when a heading it keeps is named otherwise now", () => {
    const sheet = headedSheet();
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 5],
          [7, 9],
          [11, 13],
          [15, 20],
        ],
        from: 0,
        to: 2,
        html: '<h1 id="dos" data-at="0-5">Dos</h1><p data-at="7-9">A.</p>',
        heads: ["uno"],
      }),
      0,
    );
    expect(laid).toBeNull();
    expect(sheet.querySelector("h1")?.id).toBe("uno");
  });

  it("lays a heading drawn again when those it keeps keep their names", () => {
    const sheet = headedSheet();
    sheet.lastElementChild?.setAttribute("id", "dos");
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 6],
          [8, 10],
          [12, 14],
          [16, 21],
        ],
        from: 0,
        to: 2,
        html: '<h1 id="unos" data-at="0-6">Unos</h1><p data-at="8-10">A.</p>',
        heads: ["dos"],
      }),
      1,
    );
    expect(laid).toHaveLength(2);
    expect(sheet.querySelector("h1")?.id).toBe("unos");
  });

  it("keeps the name of a heading drawn again beside the key, which the core leaves unnamed", () => {
    const sheet = headedSheet();
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 5],
          [7, 10],
          [12, 14],
          [16, 21],
        ],
        from: 0,
        to: 2,
        html:
          '<h1 data-at="0-5"><span data-src="2-5">Uno</span></h1>' +
          '<p data-at="7-10"><span data-src="7-10">Ax.</span></p>',
      }),
      1,
    );
    expect(laid).toHaveLength(2);
    expect(Array.from(sheet.querySelectorAll("h1"), (one) => one.id)).toEqual(["uno", "uno-1"]);
  });

  it("asks for the whole page when a heading comes as another goes beside the key", () => {
    document.body.innerHTML =
      '<article class="page"><blockquote data-at="0-23">' +
      '<figure class="code" data-at="2-17"></figure><h1 id="b" data-at="18-23">b</h1>' +
      '</blockquote><p data-at="25-27">A.</p></article>';
    const sheet = document.querySelector(".page") as Element;
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 22],
          [24, 26],
        ],
        from: 0,
        to: 1,
        html: '<blockquote data-at="0-22"><p data-at="2-4">~~</p><h1 data-at="7-10">a</h1></blockquote>',
      }),
      -1,
    );
    expect(laid).toBeNull();
    expect(sheet.querySelector("h1")?.id).toBe("b");
  });

  it("asks for the whole page when a heading it draws again is words now", () => {
    const sheet = headedSheet();
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 3],
          [5, 7],
          [9, 11],
          [13, 18],
        ],
        from: 0,
        to: 2,
        html: '<p data-at="0-3">Uno</p><p data-at="5-7">A.</p>',
      }),
      -2,
    );
    expect(laid).toBeNull();
  });

  it("sums heading names as the core does", () => {
    expect(namesSum(["uno", "título"])).toEqual([0x950d02f5, 0x507896ce]);
    expect(namesSum([])).toEqual([0x811c9dc5, 0x9747b28c]);
  });
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
    expect(placeOf(last)).toEqual([15, 21]);
    expect(placeOf(last?.querySelector("[data-src]"), "data-src")).toEqual([15, 21]);
    placesSettled(document);
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
    expect(placeOf(last.querySelector("[data-placed]"), "data-placed")).toEqual([15, 21]);
    placesSettled(document);
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

  it("asks for the whole page when a key defines a link or a footnote otherwise", () => {
    const sheet = sheetOf();
    const before = sheet.innerHTML;
    const defined = (whole: boolean) =>
      near({
        spans: [
          [0, 4],
          [6, 10],
          [12, 18],
        ],
        from: 2,
        to: 3,
        html: '<p class="refdef" data-at="12-18"><span class="refdef-label">[x]</span></p>',
        whole,
      });
    expect(patched(sheet, defined(true), 0)).toBeNull();
    expect(sheet.innerHTML).toBe(before);
    expect(patched(sheet, defined(false), 0)).toHaveLength(1);
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
