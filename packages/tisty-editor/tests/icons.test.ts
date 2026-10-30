import { describe, expect, it } from "vitest";
import { iconed } from "../src/icons";

const page = (html: string): Document => {
  const made = document.implementation.createHTMLDocument("");
  made.body.innerHTML = html;
  return made;
};

describe("the icons a text names", () => {
  it("draws a known icon with its hue and leaves an odd hue out", () => {
    const drawn = page(
      '<p><span class="ico" data-ico="star" data-hue="blue"></span><span class="ico" data-ico="star" data-hue="a b"></span><span class="ico" data-ico="nope"></span></p>',
    );
    iconed(drawn);
    const [blue, odd, unknown] = Array.from(drawn.querySelectorAll<HTMLElement>("span.ico"));
    expect(blue.querySelector("svg")?.getAttribute("class")).toBe("glyph");
    expect(blue.classList.contains("ico-blue")).toBe(true);
    expect(odd.querySelector("svg")).not.toBeNull();
    expect(odd.className).toBe("ico");
    expect(unknown.querySelector("svg")).toBeNull();
  });
});
