import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { DEEPEST } from "../core";

describe("how deep folders are allowed to go", () => {
  it("agrees with the core, which is the one that refuses the move", () => {
    const rust = readFileSync(
      resolve(process.cwd(), "../crates/tisty-core/src/model/folder.rs"),
      "utf8",
    );
    const said = rust.match(/pub const DEEPEST: usize = (\d+);/);

    expect(said).not.toBeNull();
    expect(Number(said?.[1])).toBe(DEEPEST);
  });
});
