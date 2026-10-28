import { describe, expect, it } from "vitest";
import { docked } from "../ui/docking";

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
