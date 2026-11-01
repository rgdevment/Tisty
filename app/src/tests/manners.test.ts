import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const TYPE = ["21", "13", "12.5", "11.5", "10.5", "9"];
const CORNERS = ["md", "full", "[10px]"];
const VEILS = ["10", "40", "70"];

const looked = ["src/ui", "src/ui/viewer", "src"];

const sources = (): { name: string; body: string }[] =>
  looked.flatMap((where) =>
    readdirSync(where, { withFileTypes: true })
      .filter((one) => one.isFile() && /\.tsx$/.test(one.name))
      .map((one) => ({
        name: `${where}/${one.name}`,
        body: readFileSync(`${where}/${one.name}`, "utf8"),
      })),
  );

const strays = (of: RegExp, allowed: string[]): string[] =>
  sources().flatMap(({ name, body }) =>
    [...body.matchAll(of)]
      .map((hit) => hit[1] ?? "bare")
      .filter((one) => !allowed.includes(one))
      .map((one) => `${name}: ${one}`),
  );

// The column beside a sheet is drawn by the editor's package from its own stylesheet, which the window loads.
const SIDE = "../packages/tisty-editor/src/side.css";
const PX = { TYPE: ["21", "13", "12.5", "11.5", "10.5", "10", "9"], CORNERS: ["6", "10", "999"] };

describe("the column's own sheet keeps to the house's scale", () => {
  const sheet = readFileSync(SIDE, "utf8");
  const off = (of: RegExp, allowed: string[]) =>
    [...sheet.matchAll(of)].map((hit) => hit[1]).filter((one) => !allowed.includes(one));

  it("sets type at the house's sizes", () => {
    expect(off(/font-size:\s*([0-9.]+)px/g, PX.TYPE)).toEqual([]);
  });

  it("rounds corners the house's ways", () => {
    expect(off(/border-radius:\s*([0-9.]+)px/g, PX.CORNERS)).toEqual([]);
  });
});

describe("the house keeps one scale", () => {
  it("sets type at one of the six sizes, and no other", () => {
    expect(strays(/\btext-\[([0-9.]+)px\]/g, TYPE)).toEqual([]);
  });

  it("never reaches for a named size, which a bracketed guard cannot see", () => {
    expect(strays(/\btext-(xs|sm|base|lg|[0-9]?xl)\b/g, [])).toEqual([]);
  });

  it("rounds a corner three ways, and no other", () => {
    expect(
      strays(/\brounded(?:-[trbl][lr]?)?-(\[[0-9]+px\]|none|sm|md|lg|xl|2xl|3xl|full)/g, CORNERS),
    ).toEqual([]);
  });

  it("never leaves a corner bare, which takes four pixels nothing else uses", () => {
    expect(strays(/\brounded(?:-[trbl][lr]?)?(?![-\w[])/g, [])).toEqual([]);
  });

  it("veils a colour at one of three strengths, and no other", () => {
    expect(
      strays(/\b(?:bg|text|border|ring|outline|fill|stroke)-[a-z-]+\/([0-9]+)/g, VEILS),
    ).toEqual([]);
  });
});
