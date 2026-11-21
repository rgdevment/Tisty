import { existsSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const holding = (mark: string): string => {
  let at = resolve(process.cwd());
  for (;;) {
    if (existsSync(join(at, mark))) return at;
    const up = dirname(at);
    if (up === at) throw new Error(`no ${mark} above ${process.cwd()}`);
    at = up;
  }
};

export const inTheRepo = (...parts: string[]): string => join(holding("crates"), ...parts);

export const windowFiles = (at = inTheRepo("app", "src")): string[] =>
  readdirSync(at, { withFileTypes: true }).flatMap((one) => {
    const here = join(at, one.name);
    if (one.isDirectory()) return one.name === "tests" ? [] : windowFiles(here);
    return /\.tsx?$/.test(one.name) ? [here] : [];
  });
