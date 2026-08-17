import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { saidPlainly } from "../refusal";

const roots = ["src-tauri/src", "../crates/tisty-core/src", "../crates/tisty-sync/src"];

const rustFiles = (at: string): string[] =>
  readdirSync(at).flatMap((one) => {
    const path = join(at, one);
    if (statSync(path).isDirectory()) return rustFiles(path);
    return path.endsWith(".rs") ? [path] : [];
  });

const emitted = (): string[] => {
  const found = new Set<string>();
  for (const root of roots) {
    for (const file of rustFiles(root)) {
      const text = readFileSync(file, "utf8");
      for (const [, code] of text.matchAll(/Refusal::(?:of|about)\(\s*"([a-zA-Z]+)"/g)) {
        found.add(code);
      }
    }
  }
  return [...found].sort();
};

describe("every refusal the core can send", () => {
  it("has a sentence the window can show", () => {
    const codes = emitted();
    expect(codes.length).toBeGreaterThan(10);

    const raw = codes.filter((code) => saidPlainly({ code }).includes(code));

    expect(raw).toEqual([]);
  });
});
