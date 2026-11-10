import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { above } from "./above";

// The window's colours live in two files: a drawn page's in tisty-sheet, the rest here.
export const themeCss = (): string =>
  [above("packages/tisty-sheet/src/tokens.css"), resolve(process.cwd(), "src/index.css")]
    .map((file) => readFileSync(file, "utf8"))
    .join("\n");

export const themed = (css: string, opener: string): string => {
  const blocks: string[] = [];
  for (let at = css.indexOf(opener); at > -1; at = css.indexOf(opener, at + opener.length))
    blocks.push(css.slice(at, css.indexOf("\n}", at)));
  return blocks.join("\n");
};
