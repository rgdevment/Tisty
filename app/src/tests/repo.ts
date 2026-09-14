import { existsSync } from "node:fs";
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
