import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// Stryker runs the tests from a copy, so a file is found beside the test rather than from where it runs.
export const source = (name: string): string =>
  readFileSync(join(dirname(fileURLToPath(import.meta.url)), "../src", name), "utf8");
