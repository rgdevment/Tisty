import { render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import Glyph from "../ui/Glyph";

const drawn = (name: string) =>
  render(<Glyph name={name} className="h-[15px] w-[15px]" />).container.firstElementChild;

const measured = (width: number) => {
  Object.defineProperty(SVGElement.prototype, "getBBox", {
    configurable: true,
    value: () => ({ x: 0, y: 0, width, height: 24 }),
  });
};

describe("an icon beside a folder", () => {
  afterEach(() => {
    delete (SVGElement.prototype as { getBBox?: unknown }).getBBox;
  });

  it("draws an emoji in the same box as an icon from the table", () => {
    const icon = drawn("home");
    for (const mark of ["🎧", "➿", "🦫"]) {
      const emoji = drawn(mark);
      expect(emoji?.tagName).toBe(icon?.tagName);
      expect(emoji?.getAttribute("viewBox")).toBe(icon?.getAttribute("viewBox"));
      expect(emoji?.getAttribute("class")).toBe(icon?.getAttribute("class"));
      expect(emoji?.textContent).toBe(mark);
    }
  });

  it("leaves an emoji that fits as it is drawn", () => {
    measured(24.7);
    expect(drawn("🎧")?.querySelector("text")?.getAttribute("textLength")).toBeNull();
  });

  it("presses in a sequence the font draws as several emoji, rather than cutting it", () => {
    measured(96);
    const text = drawn("👨‍👩‍👧‍👦")?.querySelector("text");
    expect(text?.getAttribute("textLength")).toBe("24");
    expect(text?.getAttribute("lengthAdjust")).toBe("spacingAndGlyphs");
  });

  it("draws nothing for a name it does not know", () => {
    expect(drawn("nosuchthing")).toBeNull();
  });
});
