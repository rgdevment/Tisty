import { describe, expect, it } from "vitest";
import { leftBehind } from "../leftBehind";
import { fill } from "../locales";

describe("what an import says it could not bring", () => {
  it("says nothing when everything came in", () => {
    expect(leftBehind({ missed: 0, notes: 0 })).toBe("");
  });

  it("names the files left behind and the notes that stayed as words, each once", () => {
    expect(leftBehind({ missed: 2, notes: 0 })).toBe(fill("importMissed", "2"));
    expect(leftBehind({ missed: 0, notes: 1 })).toBe(fill("importNotes", "1"));
    expect(leftBehind({ missed: 1, notes: 3 })).toBe(
      `${fill("importMissed", "1")} ${fill("importNotes", "3")}`,
    );
  });
});
