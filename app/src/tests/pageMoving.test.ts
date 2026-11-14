import { afterEach, describe, expect, it } from "vitest";
import { acted, gripped, movedTo, swappedItem, topsOf, unitOf } from "../ui/pageMoving";

const BODY = "Intro.\n\n## Uno\n\nTexto uno.\n\n## Dos\n\nTexto dos.\n\n1. primero\n2. segundo\n";
const PAGE =
  '<p data-at="0-6">Intro.</p>' +
  '<h2 data-at="8-14">Uno</h2>' +
  '<p data-at="16-26">Texto uno.</p>' +
  '<h2 data-at="28-34">Dos</h2>' +
  '<p data-at="36-46">Texto dos.</p>' +
  '<ol data-at="48-69"><li data-at="48-58">primero</li><li data-at="59-69">segundo</li></ol>';

afterEach(() => {
  document.body.innerHTML = "";
});

const sheetOf = () => {
  document.body.innerHTML = `<article class="page">${PAGE}</article>`;
  return document.querySelector(".page") as Element;
};

describe("a block moved on the page written in place", () => {
  it("carries a title's section with it, up to the next title as high", () => {
    const tops = topsOf(sheetOf());
    const unit = unitOf(BODY, tops, 1);
    expect(BODY.slice(unit?.from, unit?.to)).toBe("## Uno\n\nTexto uno.");
  });

  it("goes up past the block above, and down past the unit below", () => {
    const tops = topsOf(sheetOf());
    expect(acted(BODY, tops, 2, "up")?.text).toBe(
      "Intro.\n\nTexto uno.\n\n## Uno\n\n## Dos\n\nTexto dos.\n\n1. primero\n2. segundo\n",
    );
    expect(acted(BODY, tops, 1, "down")?.text).toBe(
      "Intro.\n\n## Dos\n\nTexto dos.\n\n1. primero\n2. segundo\n\n## Uno\n\nTexto uno.\n",
    );
    expect(acted(BODY, tops, 0, "up")).toBeNull();
  });

  it("leaves a copy under itself, or goes with the blank lines after it", () => {
    const tops = topsOf(sheetOf());
    const doubled = acted(BODY, tops, 0, "double");
    expect(doubled?.text.startsWith("Intro.\n\nIntro.\n\n## Uno")).toBe(true);
    expect(doubled?.caret).toBe(8);
    expect(acted(BODY, tops, 0, "drop")?.text.startsWith("## Uno")).toBe(true);
  });

  it("lands where it is dropped, the caret in its first words", () => {
    const tops = topsOf(sheetOf());
    const unit = unitOf(BODY, tops, 0);
    const moved = unit && movedTo(BODY, unit, BODY.indexOf("## Dos"));
    expect(moved?.text.startsWith("## Uno\n\nTexto uno.\n\nIntro.\n\n## Dos")).toBe(true);
    expect(moved?.text.slice(moved.caret, moved.caret + 6)).toBe("Intro.");
  });

  it("trades an item with the one beside it, each place keeping its number", () => {
    const sheet = sheetOf();
    const second = sheet.querySelectorAll("li")[1] as Element;
    const moved = swappedItem(BODY, second, -1);
    expect(moved?.text.endsWith("1. segundo\n2. primero\n")).toBe(true);
    expect(moved?.text.slice(moved.caret, moved.caret + 7)).toBe("segundo");
    expect(swappedItem(BODY, second, 1)).toBeNull();
  });
});

describe("the grip beside a block", () => {
  it("stays away from a sheet whose margin has no room for it", () => {
    const sheet = sheetOf();
    (sheet as HTMLElement).style.paddingLeft = "12px";
    const stop = gripped(document, { asked: () => {}, moved: () => {} });
    sheet.firstElementChild?.dispatchEvent(new MouseEvent("mousemove", { bubbles: true }));
    expect((document.querySelector(".page-grip") as HTMLElement).hidden).toBe(true);
    stop();
  });

  it("asks what to do with the block on a press, and moves it when dragged", () => {
    const sheet = sheetOf();
    (sheet as HTMLElement).style.paddingLeft = "40px";
    const asked: Element[] = [];
    const moved: [Element, Element | null][] = [];
    const stop = gripped(document, {
      asked: (top) => asked.push(top),
      moved: (top, target) => moved.push([top, target]),
    });
    const first = sheet.firstElementChild as Element;
    first.dispatchEvent(new MouseEvent("mousemove", { bubbles: true }));
    const grip = document.querySelector(".page-grip") as HTMLElement;
    expect(grip.hidden).toBe(false);
    grip.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0, clientY: 10 }));
    document.dispatchEvent(new MouseEvent("mouseup", { bubbles: true }));
    expect(asked).toEqual([first]);
    grip.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0, clientY: 10 }));
    document.dispatchEvent(new MouseEvent("mousemove", { bubbles: true, clientY: 200 }));
    document.dispatchEvent(new MouseEvent("mouseup", { bubbles: true }));
    expect(moved).toHaveLength(1);
    expect(moved[0]?.[0]).toBe(first);
    stop();
    expect(document.querySelector(".page-grip")).toBeNull();
  });
});
