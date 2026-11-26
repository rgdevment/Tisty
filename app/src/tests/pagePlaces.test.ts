import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { patched } from "../ui/pagePatching";
import {
  keyWritten,
  placeCopied,
  placedAt,
  placeOf,
  placesSettled,
  placeWritten,
} from "../ui/pagePlaces";
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

// Like the core, it draws one block on each side of those the key reached.
const nearOf = (blocks: readonly string[], from: number, now: number) => {
  const starts = startsOf(blocks);
  const first = Math.max(0, from - 1);
  const last = Math.min(blocks.length, from + now + 1);
  return near({
    spans: blocks.map((one, at): Span => [starts[at], starts[at] + one.length]),
    from: first,
    to: last,
    html: blocks
      .slice(first, last)
      .map((one, at) => drawnOne(one, starts[first + at]))
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

const endOf = (blocks: readonly string[], at: number) => startsOf(blocks)[at] + blocks[at].length;

const typed = (sheet: Element, blocks: string[], at: number, word: string) => {
  const end = endOf(blocks, at);
  blocks[at] += word;
  return patched(sheet, nearOf(blocks, at, 1), word.length, [end, end]);
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
    patched(sheet, nearOf(blocks, 0, 1), -2, [1, 3]);

    expect(placeOf(sheet.lastElementChild)).toEqual([8, 12]);
  });

  it("leaves nothing waiting when the keys come back to where they began", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "x");
    blocks[0] = "uno";
    patched(sheet, nearOf(blocks, 0, 1), -1, [3, 4]);

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

  it("writes what waits before a place is copied into a block still waiting, so it is not moved again", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "x");
    const first = sheet.firstElementChild as Element;
    const last = sheet.lastElementChild as Element;
    const card = document.createElement("span");
    last.append(card);

    placeCopied(first, card, "data-at", "data-placed");
    placesSettled(document);

    expect(card.getAttribute("data-placed")).toBe(first.getAttribute("data-at"));
    expect(last.getAttribute("data-at")).toBe("11-15");
  });

  it("leaves what waits as it is when a place is copied within the block the key drew", () => {
    const blocks = ["uno", "dos", "tres"];
    const sheet = sheetOf(blocks);
    typed(sheet, blocks, 0, "x");
    const first = sheet.firstElementChild as Element;
    const last = sheet.lastElementChild as Element;
    const card = document.createElement("span");
    first.append(card);

    placeCopied(first, card, "data-at", "data-placed");

    expect(card.getAttribute("data-placed")).toBe(first.getAttribute("data-at"));
    expect(last.getAttribute("data-at")).toBe("10-14");
    expect(placeOf(last)).toEqual([11, 15]);
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
      const block = sheet.querySelectorAll(":scope > [data-at]")[at];
      const kind = pick(10);
      let laid: Element[] | null;
      const start = startsOf(blocks)[at];
      const end = start + was.length;
      if (kind < 6) {
        const word = "abcdefgh".slice(0, 1 + pick(kind < 2 ? 8 : 1));
        blocks[at] = was + word;
        laid = patched(sheet, nearOf(blocks, at, 1), word.length, [end, end]);
      } else if (kind < 8 && was.length > 3) {
        blocks[at] = was.slice(0, -1);
        laid = patched(sheet, nearOf(blocks, at, 1), -1, [end - 1, end]);
      } else if (kind === 8 && was.length > 4) {
        const cut = 2 + pick(was.length - 3);
        blocks.splice(at, 1, was.slice(0, cut), `w${was.slice(cut)}`);
        laid = patched(sheet, nearOf(blocks, at, 2), 3, [start + cut, start + cut]);
      } else if (at + 1 < blocks.length) {
        blocks.splice(at, 2, `${was}${blocks[at + 1]}`);
        laid = patched(sheet, nearOf(blocks, at, 1), -2, [end, end + 2]);
      } else continue;
      expect(laid, `key ${key}`).not.toBeNull();
      expect(laid?.[0] === block, `key ${key} keeps its paragraph`).toBe(block?.tagName === "P");
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
    /(?<!\b(?:placeOf|placeSaid|placeCopied|placeWritten|removeAttribute)\((?:[^()]|\([^()]*\))*)["']data-(at|src|placed)["']/,
    /["'`]data-["'`]\s*\+|`data-\$\{/,
  ];
  const raw = (said: string) => RAW.some((one) => one.test(said));

  it("tells a raw read from a read through the page's places, a name kept in a constant too", () => {
    expect(raw('spanned(one.getAttribute("data-at"))')).toBe(true);
    expect(raw('const AT = "data-at";')).toBe(true);
    expect(raw("const NAMES = ['data-src', 'data-placed'];")).toBe(true);
    expect(raw("one.dataset.at")).toBe(true);
    expect(raw("page.querySelector(\".page > [data-at^='7-']\")")).toBe(true);
    expect(raw('placeOf(one, "data-src") ?? placeOf(one, "data-placed")')).toBe(false);
    expect(raw('one.getAttribute("data-" + kind)')).toBe(true);
    expect(raw(["one.getAttribute(`data-$", "{kind}`)"].join(""))).toBe(true);
    expect(raw('placeCopied(img, card, "data-src", "data-placed")')).toBe(false);
    expect(raw('placeOf(target.closest("li"), "data-src")')).toBe(false);
    expect(raw('one.removeAttribute("data-at")')).toBe(false);
    expect(raw('sheet.querySelectorAll(":scope > [data-at]")')).toBe(false);
  });

  // A mutation run copies only the app and wraps every word of the files it mutates; the plain run judges those.
  const judged = (said: string) => !said.includes("stryMutAct_") && raw(said);

  it("leaves a file a mutation run rewrote to the run that reads it as written", () => {
    const mutated = 'placeOf(one, stryMutAct_9fa48("7") ? "" : (stryCov_9fa48("7"), "data-src"))';

    expect(raw(mutated)).toBe(true);
    expect(judged(mutated)).toBe(false);
    expect(judged('spanned(one.getAttribute("data-at"))')).toBe(true);
  });

  it("never reads, writes or looks a place up by its words anywhere else", () => {
    const roots = ["src", "../packages"].filter((one) => existsSync(one));
    const guilty = roots.flatMap(walk).filter((one) => judged(readFileSync(one, "utf8")));

    expect(guilty).toEqual([]);
  });
});

describe("the places moved by a key the browser wrote", () => {
  const BOLD =
    '<p data-at="0-24"><span data-src="0-4">Uno </span><strong><span data-src="6-9">dos</span></strong><span data-src="11-16"> tres</span></p>' +
    '<p data-at="26-30"><span data-src="26-30">Dos.</span></p>' +
    '<p data-at="32-36"><span data-src="32-36">Fin.</span></p>';

  const read = (html = BOLD) => {
    document.body.innerHTML = `<article class="page">${html}</article>`;
    const [first, second, third] = Array.from(document.querySelectorAll(".page > p"));
    const spans = Array.from(first.querySelectorAll("span"));
    return { first, second, third, spans };
  };

  it("moves the end of the span and of the block that holds it, and the places after it in the block", () => {
    const { first, spans } = read();
    keyWritten(first, spans[1], 9, 2);
    expect(spans.map((one) => one.getAttribute("data-src"))).toEqual(["0-4", "6-11", "13-18"]);
    expect(first.getAttribute("data-at")).toBe("0-26");
  });

  it("leaves the places before the key alone, and the start of the span", () => {
    const { first, spans } = read();
    keyWritten(first, spans[2], 16, 1);
    expect(spans.map((one) => one.getAttribute("data-src"))).toEqual(["0-4", "6-9", "11-17"]);
    expect(first.getAttribute("data-at")).toBe("0-25");
  });

  it("moves them back for a letter taken away", () => {
    const { first, spans } = read();
    keyWritten(first, spans[1], 8, -1);
    expect(spans.map((one) => one.getAttribute("data-src"))).toEqual(["0-4", "6-8", "10-15"]);
    expect(first.getAttribute("data-at")).toBe("0-23");
  });

  it("reads the later blocks moved though their attributes wait", () => {
    const { first, second, third, spans } = read();
    keyWritten(first, spans[0], 4, 3);
    expect(second.getAttribute("data-at")).toBe("26-30");
    expect(placeOf(second)).toEqual([29, 33]);
    expect(placeOf(second.querySelector("span"), "data-src")).toEqual([29, 33]);
    expect(placeOf(third)).toEqual([35, 39]);
    expect(placeOf(first)).toEqual([0, 27]);
  });

  it("writes the later blocks when the places settle", () => {
    const { first, second, third, spans } = read();
    keyWritten(first, spans[0], 4, 3);
    placesSettled(document);
    expect(second.getAttribute("data-at")).toBe("29-33");
    expect(second.querySelector("span")?.getAttribute("data-src")).toBe("29-33");
    expect(third.getAttribute("data-at")).toBe("35-39");
  });

  it("adds up the keys that follow each other before the places settle", () => {
    const { first, second, third, spans } = read();
    keyWritten(first, spans[0], 4, 1);
    keyWritten(first, spans[0], 5, 2);
    expect(placeOf(second)).toEqual([29, 33]);
    placesSettled(document);
    expect(third.getAttribute("data-at")).toBe("35-39");
  });

  it("settles the blocks of an earlier key when the next key is in another block", () => {
    const { first, second, third, spans } = read();
    keyWritten(first, spans[0], 4, 1);
    keyWritten(second, second.querySelector("span") as Element, 30, 1);
    expect(first.getAttribute("data-at")).toBe("0-25");
    expect(second.getAttribute("data-at")).toBe("27-32");
    expect(third.getAttribute("data-at")).toBe("33-37");
    expect(placeOf(third)).toEqual([34, 38]);
  });

  it("moves nothing for a key that changed no bytes", () => {
    const { first, second, spans } = read();
    keyWritten(first, spans[0], 4, 0);
    expect(first.getAttribute("data-at")).toBe("0-24");
    expect(placeOf(second)).toEqual([26, 30]);
  });

  it("has no later block to move after the last one", () => {
    const { third } = read();
    keyWritten(third, third.querySelector("span") as Element, 36, 2);
    expect(third.getAttribute("data-at")).toBe("32-38");
    expect(placeOf(third.querySelector("span"), "data-src")).toEqual([32, 38]);
  });
});
