import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { inTheRepo, windowFiles } from "./repo";

describe("which system the window runs on", () => {
  it("is read in one place, so every part of the window agrees", () => {
    const readers = windowFiles().filter((one) =>
      /\buserAgent(Data)?\b|navigator\s*\.\s*platform\b/.test(readFileSync(one, "utf8")),
    );

    expect(readers).toEqual([inTheRepo("app", "src", "ui", "macKeys.ts")]);
  });
});
