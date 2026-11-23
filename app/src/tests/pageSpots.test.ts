import { afterEach, describe, expect, it, vi } from "vitest";
import { checkOf } from "../placing";
import { opened } from "../ui/pageNodes";
import { ghostAt, lineEdge, sourceAt, spotOf, topsAt } from "../ui/pageSpots";

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
  it("takes a place after an item's own words as their end, not as the start of the list nested in it", () => {
    sheetOf(
      '<ul data-at="0-23"><li data-at="0-23"><span data-src="2-6">uno </span><span class="tag" data-src="6-10" contenteditable="false">#eti</span>' +
        '<ul data-at="13-23"><li data-at="13-23"><span data-src="15-21">dentro</span></li></ul></li></ul>',
    );
    const item = document.querySelector("li") as Element;
    expect(sourceAt("- uno #eti\n  - dentro\n", item, 2)).toBe(10);
  });

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

  it("writes after the block before a piece of the sheet that is no block, as a note left between blocks", () => {
    const sheet = sheetOf(PAGE.replace("<ul", '<div class="said">nota</div><ul'));
    const note = sheet.querySelector(".said") as Element;
    expect(sourceAt(BODY, note, 0)).toBe(17);
    expect(sourceAt(BODY, note.firstChild as Node, 2)).toBe(17);
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

  it("lays the caret after a step's box when the step has no words yet", () => {
    const steps = "- [ ] un paso\n- [ ] \nFin";
    sheetOf(
      '<ul data-at="0-21"><li class="task" data-at="0-14"><span class="check" contenteditable="false"></span><span data-src="6-13">un paso</span></li>' +
        '<li class="task" data-at="14-21"><span class="check" contenteditable="false"></span></li></ul>' +
        '<p data-at="21-24"><span data-src="21-24">Fin</span></p>',
    );
    const empty = document.querySelectorAll("li")[1] as Element;
    expect(spotOf(document, steps, 20)).toEqual([empty, 1]);
  });

  it("writes before a tag that opens a step from the room the page leaves beside the step's box", () => {
    const steps = "- [ ] uno\n- [ ] #eti dos\n";
    sheetOf(
      '<ul data-at="0-25"><li class="task" data-at="0-10"><span class="check"></span><span data-src="6-9">uno</span></li>' +
        '<li class="task" data-at="10-25"><span class="check"></span><span class="tag" data-src="16-20" contenteditable="false">#eti</span><span data-src="20-24"> dos</span></li></ul>',
    );
    opened(document);
    const room = document.querySelector("wbr") as Element;
    const step = room.parentElement as Element;
    expect(sourceAt(steps, room, 0)).toBe(16);
    expect(spotOf(document, steps, 16)).toEqual([step, 2]);
    expect(step.childNodes[1]).toBe(room);
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

  it("lays the caret where a paragraph's words start when its first word went and left a space before them", () => {
    const body = "Antes.\n\n mundo\n";
    sheetOf(
      '<p data-at="0-7"><span data-src="0-6">Antes.</span></p><p data-at="9-15"><span data-src="9-14">mundo</span></p>',
    );
    expect(spotOf(document, body, 8)).toEqual([placed("mundo").firstChild, 0]);
  });

  it("still finds no place in spaces on a line of their own, as a paragraph begun there", () => {
    const body = "Antes.\n\n \t\n\nmundo\n";
    sheetOf(
      '<p data-at="0-7"><span data-src="0-6">Antes.</span></p><p data-at="12-18"><span data-src="12-17">mundo</span></p>',
    );
    expect(spotOf(document, body, 8)).toBeNull();
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

  it("wears the title's mark right after the properties, as the core marks the line it draws there", () => {
    const body = "---\ntitle: x\n---\n\n- uno\n";
    sheetOf(
      '<dl class="front" data-at="0-17"></dl><ul data-at="18-24"><li data-at="18-24"><span data-src="20-23">uno</span></li></ul>',
    );
    const titled = ghostAt(document, body, 17);
    expect(titled?.previousElementSibling?.matches(".front")).toBe(true);
    expect(titled?.classList.contains("title")).toBe(true);
    titled?.remove();
    expect(ghostAt(document, body, 24)?.classList.contains("title")).toBe(false);
  });

  it("stands a line begun at a section's hint after the hint, never between it and its title", () => {
    const body = "## Notas\n<!-- Lo que vayas averiguando. -->\n\n## Fin\n";
    sheetOf(
      '<h2 data-at="0-9" contenteditable="false"><span data-src="3-8">Notas</span></h2>' +
        '<div data-at="9-44" class="hint" contenteditable="false"></div>' +
        '<h2 data-at="45-52" contenteditable="false"><span data-src="48-51">Fin</span></h2>',
    );
    const ghost = ghostAt(document, body, body.indexOf("-->") + 3);
    expect(ghost?.previousElementSibling?.matches(".hint")).toBe(true);
  });

  it("stands a paragraph begun under a task's steps after their «+ Agregar un paso», a new step before it", () => {
    const body = "- [ ] uno\n\n\n## Notas\n";
    sheetOf(
      '<ul data-at="0-12"><li class="task" data-at="0-10"><span class="check" contenteditable="false"></span><span data-src="6-9">uno</span></li></ul>' +
        '<button class="add-step" data-after="0-12" contenteditable="false">+ Agregar un paso</button>' +
        '<h2 data-at="12-21" contenteditable="false"><span data-src="15-20">Notas</span></h2>',
    );
    const paragraph = ghostAt(document, body, 10, null, "\n");
    expect(paragraph?.previousElementSibling?.matches(".add-step")).toBe(true);
    paragraph?.remove();
    const step = ghostAt(document, body, 10, null, "- [ ] ");
    expect(step?.nextElementSibling?.matches(".add-step")).toBe(true);
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

describe("a line's edge on a long paragraph", () => {
  it("is found walking from the caret, not by measuring every piece of the paragraph", () => {
    const pieces = Array.from(
      { length: 400 },
      (_, n) =>
        `<span data-src="${n * 10}-${n * 10 + 5}">pala </span><span class="tag" data-src="${n * 10 + 5}-${n * 10 + 10}" contenteditable="false">#eti</span>`,
    ).join("");
    sheetOf(`<p data-at="0-4000">${pieces}</p>`);
    const block = document.querySelector("p") as Element;
    const near = block.children[block.children.length - 2].firstChild as Text;
    const made = vi.spyOn(document, "createRange");
    expect(lineEdge(block, [near, 5], [near, 5], false)).toEqual([block, block.childNodes.length]);
    expect(made.mock.calls.length).toBeLessThan(8);
    made.mockRestore();
  });

  it("measures only the pieces at the line's end once it has walked there", () => {
    const pieces = Array.from(
      { length: 400 },
      (_, n) =>
        `<span data-src="${n * 10}-${n * 10 + 5}">pala </span><span class="tag" data-src="${n * 10 + 5}-${n * 10 + 10}" contenteditable="false">#eti</span>`,
    ).join("");
    sheetOf(`<p data-at="0-4000">${pieces}</p>`);
    const block = document.querySelector("p") as Element;
    const first = block.firstElementChild?.firstChild as Text;
    const walks = vi.spyOn(document, "createTreeWalker");
    expect(lineEdge(block, [first, 1], [first, 1], false)).toBeNull();
    expect(walks.mock.calls.length).toBeLessThan(8);
    walks.mockRestore();
  });
});
