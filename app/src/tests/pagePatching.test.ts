import { afterEach, describe, expect, it } from "vitest";
import { namesSum } from "../placing";
import { patched } from "../ui/pagePatching";
import { placeOf, placesSettled } from "../ui/pagePlaces";
import { near, type Span, spansSum } from "./nearing";

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

const childrenWatched = (sheet: Element) => {
  const watch = new MutationObserver(() => {});
  watch.observe(sheet, { childList: true });
  return () => {
    const said = watch.takeRecords();
    watch.disconnect();
    return said.flatMap((one) => [...one.addedNodes, ...one.removedNodes]);
  };
};

describe("a block of words drawn again as the same kind, kept where it stands", () => {
  it("takes the new words, places and attributes, and no child of the page comes or goes", () => {
    const sheet = sheetOf();
    const two = sheet.children[1] as Element;
    two.classList.add("lately");
    const moved = childrenWatched(sheet);
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
        html: '<p data-at="6-13" dir="auto"><span data-src="6-13">Two too.</span></p>',
      }),
      3,
    );
    expect(laid).toEqual([two]);
    expect(moved()).toEqual([]);
    expect(two.outerHTML).toBe(
      '<p data-at="6-13" dir="auto"><span data-src="6-13">Two too.</span></p>',
    );
  });

  it("lays a block drawn as another kind in place of the old one", () => {
    const sheet = sheetOf();
    const two = sheet.children[1] as Element;
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 12],
          [14, 20],
        ],
        from: 1,
        to: 2,
        html: '<blockquote data-at="6-12"><p data-at="8-12"><span data-src="8-12">Two.</span></p></blockquote>',
      }),
      2,
    );
    expect(laid?.[0]).not.toBe(two);
    expect(two.isConnected).toBe(false);
    expect(Array.from(sheet.children, (one) => one.tagName)).toEqual(["P", "BLOCKQUOTE", "P"]);
  });

  it("keeps the first of two paragraphs a key splits and lays the second right after it", () => {
    const sheet = sheetOf();
    const two = sheet.children[1] as Element;
    const moved = childrenWatched(sheet);
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 8],
          [10, 12],
          [14, 20],
        ],
        from: 1,
        to: 3,
        html:
          '<p data-at="6-8"><span data-src="6-8">Tw</span></p>' +
          '<p data-at="10-12"><span data-src="10-12">o.</span></p>',
      }),
      2,
    );
    expect(laid?.[0]).toBe(two);
    expect(moved()).toEqual([laid?.[1]]);
    expect(Array.from(sheet.children, (one) => one.textContent)).toEqual([
      "One.",
      "Tw",
      "o.",
      "Three.",
    ]);
  });

  it("keeps the first of two paragraphs a key joins and takes the second out", () => {
    const sheet = sheetOf();
    const [, two, three] = Array.from(sheet.children);
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 16],
        ],
        from: 1,
        to: 2,
        html: '<p data-at="6-16"><span data-src="6-16">Two.Three.</span></p>',
      }),
      -2,
    );
    expect(laid).toEqual([two]);
    expect(three?.isConnected).toBe(false);
    expect(Array.from(sheet.children, (one) => one.textContent)).toEqual(["One.", "Two.Three."]);
  });
});

describe("the blocks the core draws beside a key, left as they were when the key never reached them", () => {
  it("lays only the block the key reached, and the one after reads its place moved", () => {
    const sheet = sheetOf();
    const [one, two, three] = Array.from(sheet.children);
    const moved = childrenWatched(sheet);
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 13],
          [15, 21],
        ],
        from: 0,
        to: 3,
        html:
          '<p data-at="0-4"><span data-src="0-4">One.</span></p>' +
          '<p data-at="6-13"><span data-src="6-13">Two too.</span></p>' +
          '<p data-at="15-21"><span data-src="15-21">Three.</span></p>',
      }),
      3,
      [10, 10],
    );
    expect(laid).toEqual([two]);
    expect(moved()).toEqual([]);
    expect(Array.from(sheet.children)).toEqual([one, two, three]);
    expect(three?.getAttribute("data-at")).toBe("12-18");
    expect(placeOf(three)).toEqual([15, 21]);
    expect(placeOf(three?.querySelector("[data-src]"), "data-src")).toEqual([15, 21]);
  });

  it("lays the first block when a key writes as many letters over it, though its place held", () => {
    const sheet = sheetOf();
    const [one, two] = Array.from(sheet.children);
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 10],
          [12, 18],
        ],
        from: 0,
        to: 2,
        html:
          '<p data-at="0-4"><span data-src="0-4">Uno.</span></p>' +
          '<p data-at="6-10"><span data-src="6-10">Two.</span></p>',
      }),
      0,
      [0, 3],
    );
    expect(laid).toEqual([one]);
    expect(one?.textContent).toBe("Uno.");
    expect(sheet.children[1]).toBe(two);
  });

  it("lays nothing for a key between two blocks, and those after still move", () => {
    const sheet = sheetOf();
    const [one, two, three] = Array.from(sheet.children);
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [7, 11],
          [13, 19],
        ],
        from: 0,
        to: 1,
        html: '<p data-at="0-4"><span data-src="0-4">One.</span></p>',
      }),
      1,
      [5, 5],
    );
    expect(laid).toEqual([]);
    expect(Array.from(sheet.children)).toEqual([one, two, three]);
    expect(placeOf(two)).toEqual([7, 11]);
    expect(placeOf(three)).toEqual([13, 19]);
  });

  it("draws again a heading beside a key in a heading, as its name hangs on those before it", () => {
    const sheet = headedSheet();
    const first = sheet.children[0];
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
          '<h1 id="uno" data-at="0-5"><span data-src="2-5">Uno</span></h1>' +
          '<p data-at="7-10"><span data-src="7-10">Ax.</span></p>',
        heads: ["uno-1"],
      }),
      1,
      [9, 9],
    );
    expect(laid).toHaveLength(2);
    expect(laid?.[0]).toBe(first);
  });

  it("leaves a block with no heading beside a key in a heading as it was", () => {
    const sheet = headedSheet();
    const [, , b, last] = Array.from(sheet.children);
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 5],
          [7, 9],
          [11, 13],
          [15, 21],
        ],
        from: 2,
        to: 4,
        html:
          '<p data-at="11-13"><span data-src="11-13">B.</span></p>' +
          '<h1 id="unox" data-at="15-21"><span data-src="17-21">Unox</span></h1>',
        heads: ["uno"],
      }),
      1,
      [20, 20],
    );
    expect(laid).toEqual([last]);
    expect(sheet.children[2]).toBe(b);
    expect(last?.id).toBe("unox");
  });
});

describe("what a key leaves and what it opens again", () => {
  it("leaves a widget beside the key as it was instead of drawing the whole page", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-4"><span data-src="0-4">One.</span></p>' +
      '<p data-at="6-10"><span data-src="6-10">Two.</span></p>' +
      '<figure class="widget" data-at="12-18"></figure></article>';
    const sheet = document.querySelector(".page") as Element;
    const [, two, widget] = Array.from(sheet.children);
    const laid = patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 13],
          [15, 21],
        ],
        from: 0,
        to: 3,
        html:
          '<p data-at="0-4"><span data-src="0-4">One.</span></p>' +
          '<p data-at="6-13"><span data-src="6-13">Two too.</span></p>' +
          '<figure class="widget" data-at="15-21"></figure>',
      }),
      3,
      [10, 10],
    );
    expect(laid).toEqual([two]);
    expect(sheet.children[2]).toBe(widget);
  });

  it("keeps open the fold being written in, not a fold the key put before it", () => {
    document.body.innerHTML =
      '<article class="page"><p data-at="0-4"><span data-src="0-4">One.</span></p>' +
      '<details data-at="6-30" open><summary>Fold</summary><p data-at="20-24"><span data-src="20-24">inne</span></p></details></article>';
    const sheet = document.querySelector(".page") as Element;
    patched(
      sheet,
      near({
        spans: [
          [0, 4],
          [6, 20],
          [22, 46],
        ],
        from: 1,
        to: 3,
        html:
          '<details data-at="6-20"><summary>New</summary></details>' +
          '<details data-at="22-46"><summary>Fold</summary><p data-at="36-40"><span data-src="36-40">inne</span></p></details>',
      }),
      16,
      [6, 6],
    );
    const [added, written] = Array.from(sheet.querySelectorAll("details"));
    expect(added?.open).toBe(false);
    expect(written?.open).toBe(true);
  });
});
