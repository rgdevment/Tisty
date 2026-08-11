import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

/// Read from disk, like the palette test: the two sides of this map are Rust
/// string literals on one side and TypeScript ones on the other, so nothing but
/// a comparison can tell that they agree. They did not — the window asked for
/// `tasks`, `archive` and `tags` while the backend produced `inbox`, `today`
/// and `folded`, and every number in the sidebar was blank for it.
const rust = readFileSync(resolve(process.cwd(), "src-tauri/src/lib.rs"), "utf8");

const made = new Set(
  [...rust.matchAll(/count\(\s*"([a-z]+)"/g), ...rust.matchAll(/counts\.insert\("([a-z]+)"/g)].map(
    (hit) => hit[1],
  ),
);

/// Every key the window reads out of `counts`. A number nobody asks for is
/// waste; a key nobody answers is a blank where a count should be.
const ASKED = ["tasks", "upcoming", "repeating", "all", "tags", "archive", "folded"];

describe("the counts the sidebar and the chips paint", () => {
  it.each(ASKED)("the backend answers «%s»", (key) => {
    expect(made.has(key)).toBe(true);
  });

  it("finds the keys at all, so an empty set cannot pass this file", () => {
    expect(made.size).toBeGreaterThanOrEqual(ASKED.length);
  });

  /// Searching has no number, and saying so here keeps the next reader from
  /// «fixing» it by adding one.
  it("leaves search without one, on purpose", () => {
    expect(made.has("search")).toBe(false);
  });
});
