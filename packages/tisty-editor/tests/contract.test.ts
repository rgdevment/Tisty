import { CONTRACTS } from "@tisty/contracts";
import { describe, expect, it } from "vitest";
import { fields } from "../src/front";
import { MILLIMETRES, offered, PRINTED } from "../src/paper";

type Sheet = {
  values: {
    millimetres: Record<string, [number, number]>;
    margin_millimetres: { x: number; y: number };
    offered: string[];
  };
};

type FrontMatter = {
  values: {
    block_bytes_at_most: number;
    properties_at_most: number;
    value_characters_at_most: number;
  };
};

const sheet = (CONTRACTS.sheet as Sheet).values;
const frontMatter = (CONTRACTS.front_matter as FrontMatter).values;

describe("the paper the window lays a sheet on is the contract's", () => {
  it("its sizes", () => {
    expect(MILLIMETRES).toEqual(sheet.millimetres);
  });

  it("its margins", () => {
    const { x, y } = sheet.margin_millimetres;
    expect(PRINTED).toBe(`margin:${y}mm ${x}mm`);
  });

  it("the papers a reader picks from", () => {
    expect(offered("a4")).toEqual(sheet.offered);
  });
});

describe("the window reads a document's properties as far as the contract says", () => {
  it("up to its bytes", () => {
    const frame = "---\ntitle: a\n#\n---\n".length;
    const sized = (bytes: number) => `---\ntitle: a\n#${"x".repeat(bytes - frame)}\n---\n`;
    const atMost = frontMatter.block_bytes_at_most;
    expect(sized(atMost)).toHaveLength(atMost);
    expect(fields(sized(atMost))).toHaveLength(1);
    expect(fields(sized(atMost + 1))).toEqual([]);
    expect(fields(sized(atMost + 1).trimEnd())).toHaveLength(1);
    expect(fields(sized(atMost + 2).trimEnd())).toEqual([]);
  });

  it("up to its properties", () => {
    const keyed = (keys: number) => {
      const others = Array.from({ length: keys - 1 }, (_, at) => `k${at}: v\n`).join("");
      return `---\ntitle: a\n${others}---\n`;
    };
    const atMost = frontMatter.properties_at_most;
    expect(fields(keyed(atMost))).toHaveLength(atMost);
    expect(fields(keyed(atMost + 1))).toEqual([]);
  });

  it("up to the characters of a value", () => {
    // One emoji is one character but two units of a string, so counting units reads less than the contract says.
    const said = (characters: number) => `---\ntitle: 😀${"x".repeat(characters - 1)}\n---\n`;
    const atMost = frontMatter.value_characters_at_most;
    expect(fields(said(atMost))).toHaveLength(1);
    expect(fields(said(atMost + 1))).toEqual([]);
  });
});
