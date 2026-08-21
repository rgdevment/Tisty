import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/index.css", "utf8");
const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));

const nightly = (): string => {
  const at = css.indexOf('\n[data-theme="dark"] {');
  expect(at).toBeGreaterThan(-1);
  return css.slice(at, css.indexOf("\n}", at));
};

const ruleFor = (selector: string): string => {
  const at = css.indexOf(selector);
  expect(at).toBeGreaterThan(-1);
  return css.slice(at, css.indexOf("}", at));
};

describe("the page the writing sits on", () => {
  it("is paper by default, whatever the theme", () => {
    expect(ruleFor(".tisty-doc {")).toMatch(/background-color:\s*var\(--tisty-sheet\)/);
  });

  it("is lighter than the desk in the dark, which is the whole point", () => {
    const dark = nightly();
    const desk = /--tisty-desk:\s*#([0-9a-f]{6})/.exec(dark)?.[1];
    const sheet = /--tisty-sheet:\s*#([0-9a-f]{6})/.exec(dark)?.[1];
    expect(desk).toBeTruthy();
    expect(sheet).toBeTruthy();

    const lit = (hex: string) => Number.parseInt(hex.slice(0, 2), 16);
    expect(lit(sheet as string)).toBeGreaterThan(lit(desk as string));
  });

  it("draws no pages at all while you write, which is the point", () => {
    expect(css).not.toContain("data-leaf");
    expect(css).not.toContain("tisty-leaf");
    expect(css).not.toContain("tisty-edge");
  });
});

describe("the page break the writer puts in", () => {
  it("leaves the desk showing instead of drawing a line", () => {
    const rule = ruleFor(".tisty-doc hr {");

    expect(rule).toMatch(/background:\s*var\(--tisty-desk\)/);
    expect(rule).toMatch(/height:\s*30px/);
  });

  it("casts the edge of each sheet into the gap", () => {
    const rule = ruleFor(".tisty-doc hr {");

    expect(rule).toMatch(/inset 0 7px 7px -7px var\(--tisty-cast\)/);
    expect(rule).toMatch(/inset 0 -7px 7px -7px var\(--tisty-cast\)/);
  });

  it("casts darker in the dark, where a faint shadow would vanish", () => {
    const lit = /--tisty-cast:\s*rgb\(0 0 0 \/ ([0-9.]+)\)/.exec(css)?.[1];
    const night = /--tisty-cast:\s*rgb\(0 0 0 \/ ([0-9.]+)\)/.exec(nightly())?.[1];

    expect(Number(night)).toBeGreaterThan(Number(lit));
  });

  it("flattens back to a hairline on paper", () => {
    const printed = css.slice(css.indexOf("@media print"));
    const rule = printed.slice(printed.indexOf(".tisty-doc hr {"));

    expect(rule.slice(0, rule.indexOf("}"))).toMatch(/box-shadow:\s*none/);
  });

  it("reaches both edges of the page", () => {
    expect(ruleFor(".tisty-doc hr {")).toMatch(/margin:\s*0 -40px/);
  });

  it("becomes a real page break on paper", () => {
    const printed = css.slice(css.indexOf("@media print"));
    const rule = printed.slice(printed.indexOf(".tisty-doc hr {"));

    expect(rule.slice(0, rule.indexOf("}"))).toMatch(/break-after:\s*page/);
  });

  it("asks the printer for a sheet of paper, not a screen", () => {
    const at = css.indexOf("@page");
    expect(at).toBeGreaterThan(-1);
    expect(css.slice(at, css.indexOf("}", at))).toMatch(/size:\s*A4/i);
  });

  it("keeps pictures and tables from being cut across two sheets", () => {
    const printed = css.slice(css.indexOf("@media print"));

    expect(printed).toMatch(/break-inside:\s*avoid/);
    expect(printed).toMatch(/break-after:\s*avoid/);
  });
});

describe("the smallest the window may be", () => {
  const main = conf.app.windows[0];

  it("opens no smaller than it is allowed to be", () => {
    expect(main.width).toBeGreaterThanOrEqual(main.minWidth);
    expect(main.height).toBeGreaterThanOrEqual(main.minHeight);
  });

  it("still fits a laptop screen of 1366 by 768", () => {
    expect(main.minWidth).toBeLessThanOrEqual(1366);
    expect(main.minHeight).toBeLessThanOrEqual(768);
  });
});

describe("the paper reaching the printer", () => {
  it("names a sheet size the printer understands", () => {
    const at = css.indexOf("@page");
    expect(css.slice(at, css.indexOf("}", at))).toMatch(/size:\s*A4/i);
  });

  it("keeps the margins the sheet already had", () => {
    const at = css.indexOf("@page");
    expect(css.slice(at, css.indexOf("}", at))).toMatch(/margin:\s*22mm 20mm/);
  });

  it("drops the screen's paper width so the sheet rules on paper", () => {
    const printed = css.slice(css.indexOf("@media print"));

    expect(printed).toMatch(/main > div \{[^}]*max-width:\s*none/);
  });
});

describe("what the sheet must not take to paper", () => {
  const printed = css.slice(css.indexOf("@media print"));

  it("lets go of the screen width, which is set inline and would otherwise win", () => {
    expect(printed).toMatch(/main > div > div \{[^}]*max-width:\s*none\s*!important/);
  });

  it("still drops the window chrome and the drag strip", () => {
    expect(printed).toContain("[data-tauri-drag-region]");
    expect(printed).toContain("[data-chrome]");
  });
});

describe("what a printed page should not look like", () => {
  const printed = css.slice(css.indexOf("@media print"));

  it("marks a link card with a rule down its side, not a box around it", () => {
    const at = printed.indexOf(".beside {");
    const rule = printed.slice(at, printed.indexOf("}", at));

    expect(rule).toMatch(/border:\s*0/);
    expect(rule).toMatch(/border-left:/);
  });

  it("drops the line under the title, which only helps on screen", () => {
    const at = printed.indexOf(":first-child {");
    const rule = printed.slice(at, printed.indexOf("}", at));

    expect(rule).toMatch(/border-bottom:\s*0/);
  });
});
