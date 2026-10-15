import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { opened } from "./mounted";

interface Case {
  name: string;
  text: string;
  canonical?: string;
  now?: string;
}

const cases: Case[] = JSON.parse(
  readFileSync(join(__dirname, "../../../fixtures/markdown.json"), "utf8"),
);

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
    const wanted = one.canonical ?? one.text;
    if (one.now === undefined) {
      it(one.name, () => {
        expect(saved(one.text)).toBe(wanted);
      });
      continue;
    }
    // Where the editor falls short, what it writes today is pinned too, so a fix shows here first.
    it(`${one.name}, as the editor writes it today`, () => {
      expect(saved(one.text)).toBe(one.now);
    });
    it.fails(`${one.name}, as it should be written`, () => {
      expect(saved(one.text)).toBe(wanted);
    });
  }
});
