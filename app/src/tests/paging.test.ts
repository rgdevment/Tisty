import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/index.css", "utf8");
const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));

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
    const dark = css.slice(css.indexOf('[data-theme="dark"]'));
    const desk = /--tisty-desk:\s*#([0-9a-f]{6})/.exec(dark)?.[1];
    const sheet = /--tisty-sheet:\s*#([0-9a-f]{6})/.exec(dark)?.[1];
    expect(desk).toBeTruthy();
    expect(sheet).toBeTruthy();

    const lit = (hex: string) => Number.parseInt(hex.slice(0, 2), 16);
    expect(lit(sheet as string)).toBeGreaterThan(lit(desk as string));
  });

  it("runs on as one sheet, with no automatic breaks to measure", () => {
    expect(css).not.toContain("[data-leaf]");
    expect(css).not.toContain("--tisty-leaf");
  });
});

describe("the page break the writer puts in", () => {
  it("leaves the desk showing instead of drawing a line", () => {
    const rule = ruleFor(".tisty-doc hr {");

    expect(rule).toMatch(/background:\s*var\(--tisty-desk\)/);
    expect(rule).toMatch(/height:\s*28px/);
  });

  it("reaches both edges of the page", () => {
    expect(ruleFor(".tisty-doc hr {")).toMatch(/margin:\s*0 -40px/);
  });

  it("paints no automatic page bands", () => {
    expect(css).not.toContain("[data-leaf]");
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
