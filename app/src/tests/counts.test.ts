import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const rust = readFileSync(resolve(process.cwd(), "src-tauri/src/lib.rs"), "utf8");

const made = new Set(
  [
    ...rust.matchAll(/count\(\s*"([a-zA-Z]+)"/g),
    ...rust.matchAll(/counts\.insert\(\s*"([a-zA-Z]+)"/g),
    // The layer counts come out of a loop over their names.
    ...rust.matchAll(/\("([a-z]+)", Reading::[A-Z][a-z]+\)/g),
  ].map((hit) => hit[1]),
);

const ASKED = [
  "tasks",
  "upcoming",
  "repeating",
  "all",
  "tags",
  "archive",
  "folded",
  "stories",
  "traces",
  "tracesTold",
];

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
