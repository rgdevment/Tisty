import { existsSync, readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { inTheRepo, windowFiles } from "./repo";

const SYSTEM_READ = /\buserAgent(Data)?\b|navigator\s*\.\s*platform\b/;

const readers = (files: string[]) =>
  files.filter((one) => SYSTEM_READ.test(readFileSync(one, "utf8")));

describe("which system the window runs on", () => {
  it("is read in one place, so every part of the window agrees", () => {
    expect(readers(windowFiles())).toEqual([inTheRepo("app", "src", "ui", "macKeys.ts")]);
  });

  it("is read in one place for the packages too, the one each of them reaches", () => {
    const packages = readdirSync(inTheRepo("packages"))
      .map((one) => inTheRepo("packages", one, "src"))
      .filter((source) => existsSync(source))
      .flatMap((source) => windowFiles(source));

    expect(readers(packages)).toEqual([
      inTheRepo("packages", "tisty-diagrams", "src", "macKeys.ts"),
    ]);
  });
});
