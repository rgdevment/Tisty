import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import Glyph from "../ui/Glyph";

const drawn = (name: string) =>
  render(<Glyph name={name} className="h-[15px] w-[15px]" />).container.firstElementChild;

describe("an icon beside a folder", () => {
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

  it("draws nothing for a name it does not know", () => {
    expect(drawn("nosuchthing")).toBeNull();
  });
});
