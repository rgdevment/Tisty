import { describe, expect, it } from "vitest";
import { worded } from "../ui/blocking";

describe("what a paste brings besides plain words", () => {
  it("is translated when it carries real structure", () => {
    expect(worded("<p>Hola <strong>mundo</strong></p>")).toBe(true);
    expect(worded("<ul><li>uno</li></ul>")).toBe(true);
    expect(worded('<a href="https://x.y">x</a>')).toBe(true);
  });

  it("is left to the plain words when it is only coloured code or nothing at all", () => {
    expect(
      worded('<div style="white-space: pre"><span style="color:#f00">let</span> a</div>'),
    ).toBe(false);
    expect(worded("<span>solo texto</span>")).toBe(false);
    expect(worded("")).toBe(false);
  });
});
