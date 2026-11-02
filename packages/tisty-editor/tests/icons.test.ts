import { describe, expect, it } from "vitest";
import { markup } from "../src/glyphs";
import { iconed } from "../src/icons";
import { SHEET_GLYPHS } from "../src/sheetGlyphs";

const page = (html: string): Document => {
  const made = document.implementation.createHTMLDocument("");
  made.body.innerHTML = html;
  return made;
};

describe("the icons a text names", () => {
  it("draws a known icon with its hue and leaves an odd hue out", async () => {
    const drawn = page(
      '<p><span class="ico" data-ico="star" data-hue="blue"></span><span class="ico" data-ico="star" data-hue="a b"></span><span class="ico" data-ico="nope"></span></p>',
    );
    await iconed(drawn);
    const [blue, odd, unknown] = Array.from(drawn.querySelectorAll<HTMLElement>("span.ico"));
    expect(blue.querySelector("svg")?.getAttribute("class")).toBe("glyph");
    expect(blue.classList.contains("ico-blue")).toBe(true);
    expect(odd.querySelector("svg")).not.toBeNull();
    expect(odd.className).toBe("ico");
    expect(unknown.querySelector("svg")).toBeNull();
  });
});

describe("the glyphs the sheet draws for itself", () => {
  it("are the very ones of the whole set, so neither drifts from the other", () => {
    for (const [name, drawn] of Object.entries(SHEET_GLYPHS)) expect(drawn).toBe(markup(name));
  });
});
