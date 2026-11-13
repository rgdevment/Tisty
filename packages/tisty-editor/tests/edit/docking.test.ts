import { describe, expect, it } from "vitest";
import { docked } from "../../src/edit/docking";

describe("a bar laid over the page", () => {
  it("ends at the right edge it is given, never past the sheet's left side", () => {
    const sheet = document.createElement("article");
    sheet.className = "page";
    document.body.append(sheet);
    sheet.getBoundingClientRect = () =>
      ({ top: 0, bottom: 1000, left: 100, right: 700, width: 600 }) as DOMRect;
    const bar = document.createElement("div");
    Object.defineProperty(bar, "offsetWidth", { value: 200 });
    Object.defineProperty(bar, "offsetHeight", { value: 30 });
    let right = 650;
    const dock = docked(
      bar,
      document.createElement("span"),
      () => 300,
      undefined,
      false,
      () => right,
    );
    expect(bar.style.left).toBe("450px");

    right = 150;
    dock.place();
    expect(bar.style.left).toBe("100px");

    const plain = document.createElement("div");
    const other = docked(plain, document.createElement("span"), () => 300);
    expect(plain.style.left).not.toBe("450px");
    dock.stop();
    other.stop();
    sheet.remove();
  });
});

describe("a folded bar beside the line being written", () => {
  it("waits in the right margin when the left one has no room, and never over the tabs", () => {
    const sheet = document.createElement("article");
    sheet.className = "page";
    sheet.style.paddingLeft = "10px";
    sheet.style.paddingRight = "40px";
    document.body.append(sheet);
    sheet.getBoundingClientRect = () =>
      ({ top: 0, bottom: 1000, left: 100, right: 700, width: 600 }) as DOMRect;
    const tabs = document.createElement("div");
    tabs.className = "cm-panels-top";
    tabs.getBoundingClientRect = () => ({ top: 0, bottom: 30 }) as DOMRect;
    Object.defineProperty(tabs, "offsetHeight", { value: 30 });
    sheet.append(tabs);
    const bar = document.createElement("div");
    bar.classList.add("folded");
    Object.defineProperty(bar, "offsetWidth", { value: 24 });
    Object.defineProperty(bar, "offsetHeight", { value: 24 });
    const dock = docked(bar, document.createElement("span"), () => 0);
    expect(bar.style.left).toBe("668px");
    expect(Number.parseFloat(bar.style.top)).toBeGreaterThanOrEqual(30);
    dock.stop();
    sheet.remove();
  });
});
