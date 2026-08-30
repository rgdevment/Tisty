import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { DEEPEST, FOLDER_NAME_AT_MOST } from "../core";

const folder = (): string =>
  readFileSync(resolve(process.cwd(), "../crates/tisty-core/src/model/folder.rs"), "utf8");

describe("how deep folders are allowed to go", () => {
  it("agrees with the core, which is the one that refuses the move", () => {
    const said = folder().match(/pub const DEEPEST: usize = (\d+);/);

    expect(said).not.toBeNull();
    expect(Number(said?.[1])).toBe(DEEPEST);
  });
});

describe("how long a folder name is allowed to be", () => {
  it("agrees with the core, which is the one that refuses the name", () => {
    const said = folder().match(/pub const FOLDER_NAME_AT_MOST: usize = (\d+);/);

    expect(said).not.toBeNull();
    expect(Number(said?.[1])).toBe(FOLDER_NAME_AT_MOST);
  });
});
