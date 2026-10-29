import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = () => readFileSync("../packages/tisty-reader/src/tables.css", "utf8");

const ruleFor = (text: string, selector: string): string => {
  const at = text.indexOf(`\n${selector} {`);
  expect(at).toBeGreaterThan(-1);
  return text.slice(at, text.indexOf("}", at));
};

const onScreen = () => css().slice(0, css().indexOf("@media print"));

describe("a table on the page", () => {
  it("never cuts a word to fit a narrow column", () => {
    expect(ruleFor(css(), "th,\ntd")).toMatch(/overflow-wrap:\s*break-word/);
    expect(onScreen()).not.toMatch(/anywhere|hyphens/);
  });

  it("keeps a row of empty cells a line tall, padding and border included", () => {
    expect(ruleFor(css(), "tr")).toMatch(/height:\s*calc\(1lh \+ 13px\)/);
  });

  it("scrolls sideways when it is wider than the sheet", () => {
    expect(ruleFor(css(), "div.table")).toMatch(/overflow-x:\s*auto/);
  });

  it("keeps every border inside its box, so a table that fits shows no scrollbar", () => {
    expect(ruleFor(css(), "table")).toMatch(/border-collapse:\s*separate/);
    expect(ruleFor(css(), "th:last-child > .col-grip")).toMatch(/right:\s*0/);
  });

  it("on paper, lies flat, and only a table wider than the paper breaks its words", () => {
    const print = css().slice(css().indexOf("@media print"));
    expect(print).toMatch(/div\.table \{\s*overflow:\s*visible/);
    expect(print).toMatch(
      /div\.table\.wide :is\(th, td\) \{\s*hyphens:\s*auto;\s*overflow-wrap:\s*anywhere/,
    );
  });

  it("goes with the page wherever the page goes, the guide too", () => {
    const paging = readFileSync("../packages/tisty-reader/src/shell.ts", "utf8");
    expect(paging).toMatch(/PAGE_CSS = `\$\{sheet\}\$\{tables\}`/);
    expect(paging).toMatch(/<style>\$\{PAGE_CSS\}/);
    const guide = readFileSync("scripts/guide.mjs", "utf8");
    expect(guide).toMatch(/PAGE_CSS: sheet/);
    expect(guide).not.toMatch(/page\.css\?raw/);
  });
});
