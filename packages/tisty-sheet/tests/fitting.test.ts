import { afterEach, describe, expect, it } from "vitest";
import { fittedIn } from "../src/fitting";

const boxed = (cols: string, room: number, laidWide = room, padding = "") => {
  document.body.innerHTML = `<div class="page"><div class="table" data-at="0-9" style="${padding}"><table><colgroup>${cols}</colgroup></table></div></div>`;
  const box = document.querySelector<HTMLElement>(".table") as HTMLElement;
  Object.defineProperty(box, "clientWidth", { get: () => room, configurable: true });
  const table = box.querySelector("table") as HTMLElement;
  Object.defineProperty(table, "offsetWidth", { get: () => laidWide, configurable: true });
  return box;
};

const widths = (box: HTMLElement) =>
  Array.from(box.querySelectorAll<HTMLElement>("col")).map((one) => one.style.width);

afterEach(() => {
  document.body.innerHTML = "";
  document.documentElement.className = "";
});

describe("a table kept on its sheet", () => {
  it("keeps the shares its rule asks for once they outgrow the sheet, past the grips' margin", () => {
    const box = boxed(
      '<col style="width:600px"><col style="width:1500px">',
      726,
      700,
      "padding-left:26px",
    );
    fittedIn(box);
    expect(widths(box)).toEqual(["200px", "500px"]);
    expect(Number(box.dataset.fit)).toBeCloseTo(1 / 3);
  });

  it("leaves a table that fits as its rule says", () => {
    const box = boxed('<col style="width:200px"><col style="width:300px">', 700);
    fittedIn(box);
    expect(widths(box)).toEqual(["200px", "300px"]);
    expect(box.dataset.fit).toBeUndefined();
  });

  it("leaves room for a column whose rule keeps no width", () => {
    const box = boxed('<col style="width:1200px"><col>', 700);
    fittedIn(box);
    expect(widths(box)).toEqual(["620px", ""]);
  });

  it("makes up for what only shows once laid: borders, padding, words that never break", () => {
    const box = boxed('<col style="width:600px"><col style="width:1500px">', 700, 760);
    fittedIn(box);
    expect(widths(box)).toEqual(["182px", "457px"]);
  });

  it("reads the rule's widths, not the ones it drew, when the sheet changes size", () => {
    const box = boxed('<col style="width:600px"><col style="width:1500px">', 700);
    fittedIn(box);
    Object.defineProperty(box, "clientWidth", { get: () => 1050 });
    fittedIn(box);
    expect(widths(box)).toEqual(["300px", "750px"]);
  });

  it("never draws a column narrower than a grip can hold", () => {
    const box = boxed('<col style="width:40px"><col style="width:4000px">', 700);
    fittedIn(box);
    expect(widths(box)[0]).toBe("40px");
  });

  it("stays still while a column's edge is being dragged", () => {
    const box = boxed('<col style="width:600px"><col style="width:1500px">', 700);
    document.documentElement.classList.add("widening");
    fittedIn(box);
    expect(widths(box)).toEqual(["600px", "1500px"]);
    expect(box.dataset.fit).toBeUndefined();
  });

  it("does nothing with a table whose rule keeps no widths, or a sheet not laid yet", () => {
    const loose = boxed("<col><col>", 700);
    fittedIn(loose);
    expect(widths(loose)).toEqual(["", ""]);
    const hidden = boxed('<col style="width:900px">', 0);
    fittedIn(hidden);
    expect(widths(hidden)).toEqual(["900px"]);
  });
});
