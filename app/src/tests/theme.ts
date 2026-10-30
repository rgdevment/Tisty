import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// The window's colours live in two files: a drawn page's in the reader, which the web page shares, the rest here.
export const themeCss = (): string =>
  ["../packages/tisty-reader/src/tokens.css", "src/index.css"]
    .map((file) => readFileSync(resolve(process.cwd(), file), "utf8"))
    .join("\n");

// Every block a theme opens, in both files, read as one.
export const themed = (css: string, opener: string): string => {
  const blocks: string[] = [];
  for (let at = css.indexOf(opener); at > -1; at = css.indexOf(opener, at + opener.length))
    blocks.push(css.slice(at, css.indexOf("\n}", at)));
  return blocks.join("\n");
};
