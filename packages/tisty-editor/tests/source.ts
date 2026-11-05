import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// Stryker runs the tests from a copy, so a file is found beside the test rather than from where it runs.
export const source = (name: string): string =>
  readFileSync(join(dirname(fileURLToPath(import.meta.url)), "../src", name), "utf8");

// A stylesheet read as a module comes back empty under vitest, so the file itself is read where the package lives.
export const sheetSource = (name: string): string =>
  readFileSync(join(dirname(createRequire(import.meta.url).resolve("tisty-sheet")), name), "utf8");
