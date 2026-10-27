import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = () => readFileSync("src/ui/tables.css", "utf8");

const ruleFor = (text: string, selector: string): string => {
  const at = text.indexOf(`\n${selector} {`);
  expect(at).toBeGreaterThan(-1);
  return text.slice(at, text.indexOf("}", at));
};

describe("a table on the page", () => {
  it("never cuts a word to fit a narrow column", () => {
    expect(ruleFor(css(), "th,\ntd")).toMatch(/overflow-wrap:\s*break-word/);
    expect(css()).not.toMatch(/(th|td)[^{]*\{[^}]*overflow-wrap:\s*anywhere/);
  });

  it("scrolls sideways when it is wider than the sheet", () => {
    expect(ruleFor(css(), "div.table")).toMatch(/overflow-x:\s*auto/);
  });

  it("keeps every border inside its box, so a table that fits shows no scrollbar", () => {
    expect(ruleFor(css(), "div.table > table")).toMatch(/border-collapse:\s*separate/);
    expect(ruleFor(css(), "th:last-child > .col-grip")).toMatch(/right:\s*0/);
  });

  it("lays out flat on paper, which cannot scroll", () => {
    const print = css().slice(css().indexOf("@media print"));
    expect(print).toMatch(/div\.table \{\s*overflow:\s*visible/);
  });

  it("goes with the page wherever the page goes", () => {
    const paging = readFileSync("src/ui/paging.ts", "utf8");
    expect(paging).toMatch(/import tables from "\.\/tables\.css\?raw"/);
    expect(paging).toMatch(/\$\{sheet\}\$\{tables\}/);
  });
});
