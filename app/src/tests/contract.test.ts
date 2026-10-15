import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { opened } from "./mounted";

interface Case {
  name: string;
  text?: string;
  file?: string;
  canonical?: string;
  now?: string;
  falls_short?: string;
}

const root = join(__dirname, "../../..");

const cases: Case[] = JSON.parse(readFileSync(join(root, "fixtures/markdown.json"), "utf8"));

const textOf = (one: Case): string =>
  one.file ? readFileSync(join(root, one.file), "utf8") : (one.text ?? "");

const saved = (text: string): string => {
  const one = opened(text);
  const out = one.markdown();
  one.shut();
  return out;
};

describe("opening and saving a document leaves its bytes alone", () => {
  it("has a contract to keep at all", () => {
    expect(cases.length).toBeGreaterThan(50);
  });

  for (const one of cases) {
    const text = textOf(one);
    const wanted = one.canonical ?? text;
    if (one.now === undefined && one.falls_short === undefined) {
      it(one.name, () => {
        expect(saved(text)).toBe(wanted);
      });
      continue;
    }
    // Where the editor falls short, what it writes today is pinned too, so a fix shows here first.
    if (one.now !== undefined) {
      it(`${one.name}, as the editor writes it today`, () => {
        expect(saved(text)).toBe(one.now);
      });
    }
    it.fails(`${one.name}, as it should be written`, () => {
      expect(saved(text)).toBe(wanted);
    });
  }
});
