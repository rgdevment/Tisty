import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const rust = readFileSync(resolve(process.cwd(), "src-tauri/src/lib.rs"), "utf8");

const made = new Set(
  [...rust.matchAll(/count\(\s*"([a-z]+)"/g), ...rust.matchAll(/counts\.insert\("([a-z]+)"/g)].map(
    (hit) => hit[1],
  ),
);

const ASKED = ["tasks", "upcoming", "repeating", "all", "tags", "archive", "folded"];

describe("the counts the sidebar and the chips paint", () => {
  it.each(ASKED)("the backend answers «%s»", (key) => {
    expect(made.has(key)).toBe(true);
  });

  it("finds the keys at all, so an empty set cannot pass this file", () => {
    expect(made.size).toBeGreaterThanOrEqual(ASKED.length);
  });

  it("leaves search without one, on purpose", () => {
    expect(made.has("search")).toBe(false);
  });
});
