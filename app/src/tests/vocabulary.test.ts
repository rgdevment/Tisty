import { readFileSync } from "node:fs";
import { join } from "node:path";
import { known, markup } from "tisty-editor/read";
import { describe, expect, it } from "vitest";

const vocab = readFileSync(join(__dirname, "../../../crates/tisty-markdown/src/vocab.rs"), "utf8");
const listed = (from: string, to: string) =>
  [...vocab.slice(vocab.indexOf(from), vocab.indexOf(to)).matchAll(/"([a-z0-9-]+)"/g)].map(
    (one) => one[1],
  );

describe("the icons a document may name", () => {
  it("are exactly the ones the page draws, so the editor and the page never disagree", () => {
    const icons = listed("pub const ICONS", "pub const YELLOW");
    expect(icons.length).toBeGreaterThan(1000);
    expect(icons.filter((one) => !known(one))).toEqual([]);
    expect(known("bullets")).toBe(false);
    expect(known("dog")).toBe(true);
    expect(markup("bullets")).not.toBeNull();
  });
});
