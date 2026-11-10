import { existsSync } from "node:fs";
import { dirname, join } from "node:path";

// Stryker runs the tests from a copy of the app, so what lives beside the app is looked for up the folders.
export const above = (path: string, from = process.cwd()): string => {
  const here = join(from, path);
  if (existsSync(here)) return here;
  const up = dirname(from);
  if (up === from) throw new Error(`${path} is not above the app`);
  return above(path, up);
};
