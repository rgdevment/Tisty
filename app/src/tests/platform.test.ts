import { existsSync, readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { inTheRepo, windowFiles } from "./repo";

const SYSTEM_READ = /\buserAgent(Data)?\b|navigator\s*\.\s*platform\b/;

const readers = (files: string[]) =>
  files.filter((one) => SYSTEM_READ.test(readFileSync(one, "utf8")));

const packageFiles = () =>
  readdirSync(inTheRepo("packages"))
    .map((one) => inTheRepo("packages", one, "src"))
    .filter((source) => existsSync(source))
    .flatMap((source) => windowFiles(source));

describe("which system the window runs on", () => {
  it("is read in one place, the packages' own, so the window and every package agree", () => {
    expect(readers([...windowFiles(), ...packageFiles()])).toEqual([
      inTheRepo("packages", "tisty-diagrams", "src", "macKeys.ts"),
    ]);
  });

  it("reaches the window as the very keys the packages answer", async () => {
    const app = await import("../ui/macKeys");
    const shared = await import("tisty-diagrams/macKeys");
    expect(app.modKey).toBe(shared.modKey);
    expect(app.letterOf).toBe(shared.letterOf);
    expect(app.shortcut).toBe(shared.shortcut);
  });
});
