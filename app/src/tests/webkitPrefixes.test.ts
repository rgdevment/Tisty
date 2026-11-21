import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const PACKAGES = resolve(SRC, "..", "..", "packages");
const SOURCES = [
  join(SRC, "ui"),
  join(PACKAGES, "tisty-sheet", "src"),
  join(PACKAGES, "tisty-editor", "src"),
  join(PACKAGES, "tisty-editor", "src", "edit"),
];

const rawSheets = (): Map<string, string> => {
  const found = new Map<string, string>();
  for (const folder of SOURCES)
    for (const name of readdirSync(folder).filter((one) => /\.tsx?$/.test(one)))
      for (const [, spec] of readFileSync(join(folder, name), "utf8").matchAll(
        /from "(\.[^"]+\.css)\?raw"/g,
      )) {
        const at = resolve(folder, spec);
        found.set(at, readFileSync(at, "utf8"));
      }
  return found;
};

const declared = (css: string, property: string) =>
  [...css.matchAll(new RegExp(`(?<![\\w-])${property}\\s*:\\s*([^;}]+)`, "g"))].map((one) =>
    one[1].replace(/\s+/g, " ").trim(),
  );

describe("the rules a page carries as they are written, with no build to add Safari's names", () => {
  const sheets = rawSheets();

  it("are found where the page and its bar take them", () => {
    const names = [...sheets.keys()].map((one) => one.split("/").pop());
    expect(names).toEqual(expect.arrayContaining(["page.css", "tables.css", "taskpage.css"]));
    expect(names).toContain("bar.css");
  });

  it.each(["user-select", "backdrop-filter"])(
    "give Safari its own name for %s wherever the plain one is written",
    (property) => {
      for (const [at, css] of sheets)
        expect(declared(css, `-webkit-${property}`), `${at}: ${property}`).toEqual(
          declared(css, property),
        );
    },
  );

  it("give Safari a wavy underline of its own, which it reads only under its name", () => {
    for (const [at, css] of sheets) {
      const wavy = declared(css, "text-decoration").filter((one) => one.includes("wavy"));
      expect(declared(css, "-webkit-text-decoration"), at).toEqual(wavy);
    }
  });
});
