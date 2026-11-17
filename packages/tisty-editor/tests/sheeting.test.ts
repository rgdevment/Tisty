import { describe, expect, it, vi } from "vitest";
import { sheeted, sheeting } from "../src/sheeting";

const laid = (blocks: [string, number, number][]) => {
  document.body.innerHTML = `<article class="page">${blocks.map(([tag]) => (tag === "hr" ? "<hr>" : `<${tag}></${tag}>`)).join("")}</article>`;
  const sheet = document.querySelector<HTMLElement>(".page") as HTMLElement;
  Object.defineProperty(sheet, "offsetTop", { value: 0 });
  // The page has no layout in the test window, so each block says where it sits and how tall it is.
  Array.from(sheet.children).forEach((one, at) => {
    Object.defineProperty(one, "offsetTop", { get: () => blocks[at][1] + turnsBefore(one) });
    Object.defineProperty(one, "offsetHeight", { value: blocks[at][2] });
  });
  return sheet;
};

const turnsBefore = (one: Element) => {
  let gap = 0;
  for (let at = one.previousElementSibling; at; at = at.previousElementSibling) {
    if (at.classList.contains("sheet-turn")) gap += 200;
  }
  return gap;
};

describe("sheets laid on the page", () => {
  it("starts a block that would cross the foot of a sheet on the next one", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["p", 60, 40],
      ["p", 110, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll(".sheet-turn");
    expect(turns).toHaveLength(1);
    expect(turns[0].nextElementSibling).toBe(sheet.children[3]);
    expect(sheet.style.getPropertyValue("--last")).not.toBe("");
  });

  it("fills a sheet from the blocks in the flow, while a floated note still ends the page", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["blockquote", 50, 80],
      ["hr", 52, 2],
      ["p", 70, 20],
      ["blockquote", 90, 100],
    ]);
    for (const at of [1, 4]) {
      const note = sheet.children[at] as HTMLElement;
      note.dataset.callout = "idea";
      note.dataset.side = "right";
    }
    sheeted(document, { wide: 794, tall: 200, x: 10, y: 10 });
    expect((sheet.children[2] as HTMLElement).style.getPropertyValue("--fill")).toBe("139px");
    expect(sheet.style.getPropertyValue("--last")).toBe("53px");
  });

  it("does not take the hidden properties for the block that stays, which would leave a blank sheet", () => {
    const sheet = laid([
      ["dl", 10, 0],
      ["table", 10, 500],
    ]);
    sheet.children[0].className = "front";
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    expect(sheet.querySelectorAll(".sheet-turn")).toHaveLength(0);
  });

  it("leaves a block taller than a sheet where it starts, instead of a blank sheet before it", () => {
    const sheet = laid([
      ["table", 20, 500],
      ["p", 530, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll(".sheet-turn");
    expect(turns).toHaveLength(1);
    expect(turns[0].nextElementSibling?.tagName).toBe("P");
  });

  it("takes a heading left at the foot of a sheet to the next one, with what it heads", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["h2", 60, 30],
      ["h3", 90, 20],
      ["p", 110, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll<HTMLElement>(".sheet-turn");
    expect(turns).toHaveLength(1);
    expect(turns[0].nextElementSibling?.tagName).toBe("H2");
    expect(turns[0].style.getPropertyValue("--fill")).toBe("79px");
  });

  it("leaves a heading that already opens its sheet where it is", () => {
    const sheet = laid([
      ["p", 10, 120],
      ["h2", 130, 30],
      ["p", 160, 200],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    const turns = sheet.querySelectorAll(".sheet-turn");
    expect(turns).toHaveLength(2);
    expect(turns[0].nextElementSibling?.tagName).toBe("H2");
    expect(turns[1].nextElementSibling?.tagName).toBe("P");
  });

  it("lets a manual break end its sheet and lays nothing while a block is being written", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["hr", 60, 50],
      ["p", 120, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    expect(sheet.querySelectorAll(".sheet-turn")).toHaveLength(0);
    expect((sheet.children[1] as HTMLElement).style.getPropertyValue("--fill")).toBe("79px");
    sheet.insertAdjacentHTML("beforeend", '<div class="editing"></div>');
    (sheet.children[1] as HTMLElement).style.removeProperty("--fill");
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    expect((sheet.children[1] as HTMLElement).style.getPropertyValue("--fill")).toBe("");
  });

  it("takes the turns, the break labels and the last fill away when the sheets are turned off", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["hr", 60, 50],
      ["p", 120, 40],
      ["p", 170, 40],
    ]);
    let asked: { wide: number; tall: number; x: number; y: number; said?: string } | undefined = {
      wide: 794,
      tall: 140,
      x: 10,
      y: 10,
      said: "Salto de página",
    };
    const sheets = sheeting(document, () => asked);
    expect(sheet.querySelectorAll(".sheet-turn, .break-said").length).toBeGreaterThan(0);
    asked = undefined;
    sheets.lay();
    expect(sheet.querySelectorAll(".sheet-turn, .break-said")).toHaveLength(0);
    expect(sheet.style.getPropertyValue("--last")).toBe("");
    expect((sheet.children[1] as HTMLElement).style.getPropertyValue("--fill")).toBe("");
    expect(document.documentElement.classList.contains("sheeted")).toBe(false);
    sheets.stop();
  });

  it("keeps the first block in sight where it was while the sheets are laid again", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["hr", 60, 2],
      ["p", 120, 40],
    ]);
    const root = document.documentElement;
    root.classList.remove("sheeted");
    const box = (top: number, height: number) => ({ top, bottom: top + height }) as DOMRect;
    sheet.children[0].getBoundingClientRect = () => box(-100, 40);
    // The browser's anchoring takes the fills put back for a move and scrolls past the break.
    sheet.children[1].getBoundingClientRect = () =>
      box(root.classList.contains("sheeted") ? -552 : 200, 2);
    const scrolled = vi.spyOn(window, "scrollBy").mockImplementation(() => {});
    const sheets = sheeting(document, () => ({ wide: 794, tall: 140, x: 10, y: 10 }));
    expect(scrolled).toHaveBeenCalledWith(0, -752);
    sheets.stop();
    scrolled.mockRestore();
  });
});
