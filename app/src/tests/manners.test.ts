import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const TYPE = ["21", "13", "12.5", "11.5", "10.5", "9"];
const CORNERS = ["md", "full", "[10px]"];
const VEILS = ["10", "40", "70"];

const looked = ["src/ui", "src"];

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
      .map((hit) => hit[1])
      .filter((one) => !allowed.includes(one))
      .map((one) => `${name}: ${one}`),
  );

describe("the house keeps one scale", () => {
  it("sets type at one of the six sizes, and no other", () => {
    expect(strays(/text-\[([0-9.]+)px\]/g, TYPE)).toEqual([]);
  });

  it("rounds a corner three ways, and no other", () => {
    expect(
      strays(/\brounded(?:-[trbl][lr]?)?-(\[[0-9]+px\]|none|sm|md|lg|xl|2xl|3xl|full)/g, CORNERS),
    ).toEqual([]);
  });

  it("veils a colour at one of three strengths, and no other", () => {
    expect(
      strays(/\b(?:bg|text|border|ring|outline|fill|stroke)-[a-z-]+\/([0-9]+)/g, VEILS),
    ).toEqual([]);
  });
});
