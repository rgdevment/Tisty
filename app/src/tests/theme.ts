import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

// Stryker runs the tests from a copy of the app, so the reader's tokens are looked for up the folders.
const tokensAt = (from: string): string => {
  const here = join(from, "packages", "tisty-reader", "src", "tokens.css");
  if (existsSync(here)) return here;
  const up = dirname(from);
  if (up === from) throw new Error("packages/tisty-reader is not above the app");
  return tokensAt(up);
};

// The window's colours live in two files: a drawn page's in the reader, which the web page shares, the rest here.
export const themeCss = (): string =>
  [tokensAt(process.cwd()), resolve(process.cwd(), "src/index.css")]
    .map((file) => readFileSync(file, "utf8"))
    .join("\n");

export const themed = (css: string, opener: string): string => {
  const blocks: string[] = [];
  for (let at = css.indexOf(opener); at > -1; at = css.indexOf(opener, at + opener.length))
    blocks.push(css.slice(at, css.indexOf("\n}", at)));
  return blocks.join("\n");
};
