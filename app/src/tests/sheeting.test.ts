import { describe, expect, it } from "vitest";
import { paperIn, withPaper } from "../front";
import { sheeted } from "../ui/sheeting";

describe("the paper a document asks for", () => {
  it("reads it from the front matter, and A4 when it says nothing", () => {
    expect(paperIn("---\npaper: Letter\n---\n# Plan\n")).toBe("letter");
    expect(paperIn("---\npaper: legal\n---\n# Plan\n")).toBeNull();
    expect(paperIn("# Plan\n")).toBeNull();
  });

  it("writes another paper down and takes A4 away, keeping the rest as it was", () => {
    expect(withPaper("# Plan\n", "letter")).toBe("---\npaper: letter\n---\n# Plan\n");
    expect(withPaper("---\npaper: letter\n---\n# Plan\n", "a4")).toBe("# Plan\n");
    expect(withPaper("---\ntitle: Plan\npaper: letter\n---\nhola\n", "tabloid")).toBe(
      "---\ntitle: Plan\npaper: tabloid\n---\nhola\n",
    );
    expect(withPaper("---\ntitle: Plan\npaper: letter\n---\nhola\n", "a4")).toBe(
      "---\ntitle: Plan\n---\nhola\n",
    );
    expect(withPaper("# Plan\n", "a4")).toBe("# Plan\n");
    expect(withPaper("# Plan\r\nhola\r\n", "letter")).toBe(
      "---\r\npaper: letter\r\n---\r\n# Plan\r\nhola\r\n",
    );
  });
});

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

  it("lets a manual break end its sheet and lays nothing while a block is being written", () => {
    const sheet = laid([
      ["p", 10, 40],
      ["hr", 60, 50],
      ["p", 120, 40],
    ]);
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    expect(sheet.querySelectorAll(".sheet-turn")).toHaveLength(0);
    expect((sheet.children[1] as HTMLElement).style.getPropertyValue("--fill")).toBe("70px");
    sheet.insertAdjacentHTML("beforeend", '<div class="editing"></div>');
    (sheet.children[1] as HTMLElement).style.removeProperty("--fill");
    sheeted(document, { wide: 794, tall: 140, x: 10, y: 10 });
    expect((sheet.children[1] as HTMLElement).style.getPropertyValue("--fill")).toBe("");
  });
});
