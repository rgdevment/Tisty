import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { frail } from "../frail";

interface Case {
  text: string;
  why: string[];
}

const SAID: Record<string, string> = {
  front: "frailFront",
  html: "frailHtml",
  comments: "frailComments",
  entities: "frailEntities",
  maths: "frailMaths",
  notes: "frailNotes",
  refs: "frailRefs",
  fence: "frailFence",
  block: "frailBlocked",
};

const corpus: Case[] = JSON.parse(
  readFileSync(join(__dirname, "../../../fixtures/frail.json"), "utf8"),
);

describe("both halves read the same corpus the same way", () => {
  it("has a corpus to read at all", () => {
    expect(corpus.length).toBeGreaterThan(50);
  });

  it.each(corpus.map((one) => [one.text, one.why] as const))("agrees on %j", (text, why) => {
    expect(frail(text)).toEqual(why.map((one) => SAID[one]));
  });
});
