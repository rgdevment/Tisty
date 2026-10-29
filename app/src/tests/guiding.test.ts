import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("the guide's pages", () => {
  it("wear the page's own CSS, tables included, as the reader hands it", () => {
    const guide = readFileSync("scripts/guide.mjs", "utf8");
    expect(guide).toMatch(/PAGE_CSS: sheet/);
    expect(guide).not.toMatch(/page\.css\?raw/);
  });
});
