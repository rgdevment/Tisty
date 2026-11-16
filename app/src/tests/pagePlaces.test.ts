import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { patched } from "../ui/pagePatching";
import { placeCopied, placedAt, placeOf, placesSettled, placeWritten } from "../ui/pagePlaces";
import { near, type Span } from "./nearing";

// A toy core: blocks apart by a blank line, a quote holding its paragraph, a picture with a card for it.
const wordsOf = (said: string, at: number) =>
  Array.from({ length: Math.ceil(said.length / 4) }, (_, nth) => {
    const from = at + nth * 4;
    const to = Math.min(from + 4, at + said.length);
    return `<span data-src="${from}-${to}">${said.slice(nth * 4, nth * 4 + 4)}</span>`;
  }).join("");

const drawnOne = (said: string, at: number): string => {
  const end = at + said.length;
  if (said.startsWith("> "))
    return `<blockquote data-at="${at}-${end}"><p data-at="${at + 2}-${end}">${wordsOf(said.slice(2), at + 2)}</p></blockquote>`;
  if (said.startsWith("!"))
    return `<p data-at="${at}-${end}"><img data-src="${at}-${end}"><span data-placed="${at}-${end}"></span></p>`;
  return `<p data-at="${at}-${end}">${wordsOf(said, at)}</p>`;
};

const startsOf = (blocks: readonly string[]) => {
  let at = 0;
  return blocks.map((one) => {
    const start = at;
    at += one.length + 2;
    return start;
  });
};

const drawn = (blocks: readonly string[]) =>
  blocks.map((one, at) => drawnOne(one, startsOf(blocks)[at])).join("");

const nearOf = (blocks: readonly string[], from: number, now: number) => {
  const starts = startsOf(blocks);
  return near({
    spans: blocks.map((one, at): Span => [starts[at], starts[at] + one.length]),
    from,
    to: from + now,
    html: blocks
      .slice(from, from + now)
      .map((one, at) => drawnOne(one, starts[from + at]))
      .join(""),
  });
};

const NAMES = ["data-at", "data-src", "data-placed"] as const;

const placesRead = (root: Element) =>
  Array.from(root.querySelectorAll("[data-at], [data-src], [data-placed]")).flatMap((one) =>
    NAMES.filter((name) => one.hasAttribute(name)).map((name) => `${name}=${placeOf(one, name)}`),
  );

const placesWritten = (root: Element) =>
  Array.from(root.querySelectorAll("[data-at], [data-src], [data-placed]")).flatMap((one) =>
    NAMES.filter((name) => one.hasAttribute(name)).map(
      (name) => `${name}=${one.getAttribute(name)?.replace("-", ",")}`,
    ),
  );

const sheetOf = (blocks: readonly string[]) => {
  document.body.innerHTML = `<article class="page">${drawn(blocks)}</article>`;
  return document.querySelector(".page") as Element;
};

const wholeOf = (blocks: readonly string[]) => {
  const page = document.createElement("article");
  page.innerHTML = drawn(blocks);
  return page;
};

const typed = (sheet: Element, blocks: string[], at: number, word: string) => {
  blocks[at] += word;
  return patched(sheet, nearOf(blocks, at, 1), word.length);
};

afterEach(() => {
  placesSettled(document);
  document.body.innerHTML = "";
});

describe("the places a key moves, written once the keys rest", () => {
  it("leaves the blocks after a key as they say, and reads them moved", () => {
    const blocks = ["uno", "dos", "> tres", "!cuatro"];
    const sheet = sheetOf(blocks);
    const last = sheet.lastElementChild as Element;

    expect(typed(sheet, blocks, 1, "xy")).toHaveLength(1);

    expect(last.getAttribute("data-at")).toBe("18-25");
    expect(placeOf(last)).toEqual([20, 27]);
    expect(placesRead(sheet)).toEqual(placesWritten(wholeOf(blocks)));
  });

  it("writes them as a whole drawing would once they rest", () => {
    const blocks = ["uno", "dos", "> tres", "!cuatro"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 1, "x");
    typed(sheet, blocks, 1, "y");

    placesSettled(document);

    expect(sheet.innerHTML).toBe(wholeOf(blocks).innerHTML);
  });

  it("adds a key in the same block to what waits, without writing a place after it", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    const last = sheet.lastElementChild as Element;
    typed(sheet, blocks, 1, "x");
    typed(sheet, blocks, 1, "y");
    typed(sheet, blocks, 1, "z");

    expect(last.getAttribute("data-at")).toBe("10-14");
    expect(placeOf(last)).toEqual([13, 17]);
  });

  it("writes what waits before a key in a block after it, so neither is moved twice", () => {
    const blocks = ["uno", "dos", "tres", "cuatro"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "x");

    typed(sheet, blocks, 2, "y");

    expect(sheet.children[1].getAttribute("data-at")).toBe("6-9");
    expect(placesRead(sheet)).toEqual(placesWritten(wholeOf(blocks)));
    placesSettled(document);
    expect(sheet.innerHTML).toBe(wholeOf(blocks).innerHTML);
  });

  it("keeps reading a block laid by an earlier key as it says when the next key is in the block after", () => {
    const blocks = ["uno", "dos", "tres", "cuatro"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "abcdefghijklmnop");

    typed(sheet, blocks, 1, "x");

    expect(sheet.children[2].getAttribute("data-at")).toBe("10-14");
    expect(placesRead(sheet)).toEqual(placesWritten(wholeOf(blocks)));
  });

  it("counts back a key that takes words away", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    blocks[0] = "u";
    patched(sheet, nearOf(blocks, 0, 1), -2);

    expect(placeOf(sheet.lastElementChild)).toEqual([8, 12]);
  });

  it("leaves nothing waiting when the keys come back to where they began", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "x");
    blocks[0] = "uno";
    patched(sheet, nearOf(blocks, 0, 1), -1);

    expect(sheet.lastElementChild?.getAttribute("data-at")).toBe("10-14");
    expect(placeOf(sheet.lastElementChild)).toEqual([10, 14]);
    placesSettled(document);
    expect(sheet.innerHTML).toBe(wholeOf(blocks).innerHTML);
  });

  it("reads a page drawn whole as it says, whatever a key left waiting", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "x");

    sheet.innerHTML = drawn(blocks);

    expect(placeOf(sheet.lastElementChild)).toEqual([11, 15]);
    placesSettled(document);
    expect(sheet.innerHTML).toBe(wholeOf(blocks).innerHTML);
  });

  it("finds a block by where it stands now, not by what it still says", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "xx");

    expect(placedAt(document, ".page > *", [12, 16])).toBe(sheet.lastElementChild);
    expect(placedAt(document, ".page > *", 7)).toBe(sheet.children[1]);
  });

  it("writes what waits before a place is written or copied, so none is moved twice", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "x");
    const last = sheet.lastElementChild as Element;
    const card = document.createElement("span");

    placeCopied(last, card, "data-at", "data-placed");
    placeWritten(last, [11, 16]);

    expect(card.getAttribute("data-placed")).toBe("11-15");
    expect(placeOf(last)).toEqual([11, 16]);
  });
});

describe("random keys against the whole drawing", () => {
  const random = (seed: number) => {
    let state = seed;
    return (below: number) => {
      state = (state * 1103515245 + 12345) % 2147483648;
      return state % below;
    };
  };

  it.each([1, 2, 3, 4, 5, 6, 7, 8])("seed %i", (seed) => {
    const pick = random(seed);
    const blocks = Array.from({ length: 12 }, (_, at) =>
      ["palabra", "> cita", "!foto"][at % 3].repeat(1 + (at % 2)),
    );
    const sheet = sheetOf(blocks);
    let at = 0;
    let rested = 0;
    for (let key = 0; key < 400; key++) {
      const going = pick(8);
      if (going === 0) at = pick(blocks.length);
      else if (going === 1) at = Math.min(at + 1, blocks.length - 1);
      const was = blocks[at];
      const kind = pick(10);
      let laid: Element[] | null;
      if (kind < 6) {
        const word = "abcdefgh".slice(0, 1 + pick(kind < 2 ? 8 : 1));
        blocks[at] = was + word;
        laid = patched(sheet, nearOf(blocks, at, 1), word.length);
      } else if (kind < 8 && was.length > 3) {
        blocks[at] = was.slice(0, -1);
        laid = patched(sheet, nearOf(blocks, at, 1), -1);
      } else if (kind === 8 && was.length > 4) {
        const cut = 2 + pick(was.length - 3);
        blocks.splice(at, 1, was.slice(0, cut), `w${was.slice(cut)}`);
        laid = patched(sheet, nearOf(blocks, at, 2), 3);
      } else if (at + 1 < blocks.length) {
        blocks.splice(at, 2, `${was}${blocks[at + 1]}`);
        laid = patched(sheet, nearOf(blocks, at, 1), -2);
      } else continue;
      expect(laid, `key ${key}`).not.toBeNull();
      expect(placesRead(sheet), `key ${key}`).toEqual(placesWritten(wholeOf(blocks)));
      if (pick(12) === 0) {
        placesSettled(document);
        rested += 1;
        expect(placesWritten(sheet), `key ${key} rested`).toEqual(placesWritten(wholeOf(blocks)));
      }
    }
    placesSettled(document);
    expect(sheet.innerHTML).toBe(wholeOf(blocks).innerHTML);
    expect(rested).toBeGreaterThan(20);
  });
});

describe("the places a key moves, read only through the page's places", () => {
  const walk = (at: string): string[] =>
    readdirSync(at, { withFileTypes: true }).flatMap((one) => {
      const here = join(at, one.name);
      if (one.isDirectory())
        return ["tests", "node_modules", "dist"].includes(one.name) ? [] : walk(here);
      return /\.tsx?$/.test(one.name) && one.name !== "pagePlaces.ts" ? [here] : [];
    });

  const RAW = [
    /dataset\s*\.\s*(at|src|placed)\b/,
    /\[data-(at|src|placed)\s*[\^$*~|]?=/,
    /(?<!\b(?:placeOf|placeSaid|placeCopied|placeWritten|removeAttribute)\([^()]*)["']data-(at|src|placed)["']/,
  ];
  const raw = (said: string) => RAW.some((one) => one.test(said));

  it("tells a raw read from a read through the page's places, a name kept in a constant too", () => {
    expect(raw('spanned(one.getAttribute("data-at"))')).toBe(true);
    expect(raw('const AT = "data-at";')).toBe(true);
    expect(raw("const NAMES = ['data-src', 'data-placed'];")).toBe(true);
    expect(raw("one.dataset.at")).toBe(true);
    expect(raw("page.querySelector(\".page > [data-at^='7-']\")")).toBe(true);
    expect(raw('placeOf(one, "data-src") ?? placeOf(one, "data-placed")')).toBe(false);
    expect(raw('placeCopied(img, card, "data-src", "data-placed")')).toBe(false);
    expect(raw('one.removeAttribute("data-at")')).toBe(false);
    expect(raw('sheet.querySelectorAll(":scope > [data-at]")')).toBe(false);
  });

  it("never reads, writes or looks a place up by its words anywhere else", () => {
    // A mutation run copies only the app, so the packages are read only where they are.
    const roots = ["src", "../packages"].filter((one) => existsSync(one));
    const guilty = roots.flatMap(walk).filter((one) => raw(readFileSync(one, "utf8")));

    expect(guilty).toEqual([]);
  });
});
