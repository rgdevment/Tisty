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

describe("plain words that carry a shape of their own", () => {
  it("are looked at when they draw a table or read like code", async () => {
    const { shaped } = await import("../ui/blocking");
    expect(shaped("┌──┐\n│ a │\n└──┘")).toBe(true);
    expect(shaped("fn a() {\n  b();\n}")).toBe(true);
    expect(shaped("hola\nqué tal\nchao")).toBe(false);
    expect(shaped("una sola línea con {llaves}")).toBe(false);
  });
});

describe("what a paste from a document editor brings", () => {
  it("keeps its structure when its text wraps the way Docs and Gmail write it", async () => {
    const { worded } = await import("../ui/blocking");
    expect(worded('<p style="white-space:pre-wrap"><b>Hola</b></p>')).toBe(true);
    expect(worded('<p style="white-space: pre-line"><b>Hola</b></p>')).toBe(true);
  });

  it("leaves the drawing of a folder tree alone", async () => {
    const { shaped } = await import("../ui/blocking");
    expect(shaped(".\n├── src\n└── Cargo.toml")).toBe(false);
  });
});
