import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { themeCss, themed } from "./theme";

const css = themeCss();
const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));

const nightly = (): string => {
  const dark = themed(css, '[data-theme="dark"] {');
  expect(dark).not.toBe("");
  return dark;
};

describe("the page the writing sits on", () => {
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

describe("the paper around the writing", () => {
  it("casts darker in the dark, where a faint shadow would vanish", () => {
    const lit = /--tisty-cast:\s*rgb\(0 0 0 \/ ([0-9.]+)\)/.exec(css)?.[1];
    const night = /--tisty-cast:\s*rgb\(0 0 0 \/ ([0-9.]+)\)/.exec(nightly())?.[1];

    expect(Number(night)).toBeGreaterThan(Number(lit));
  });

  it("asks the printer for a sheet of paper, not a screen", () => {
    const at = css.indexOf("@page");
    expect(at).toBeGreaterThan(-1);
    expect(css.slice(at, css.indexOf("}", at))).toMatch(/size:\s*A4/i);
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

  it("measures the sidebar the same width the grid actually draws it", () => {
    const app = readFileSync("src/App.tsx", "utf8");
    const paper = readFileSync("src/ui/Docs.tsx", "utf8");
    const drawn =
      /grid-template-columns:(\d+)px_minmax\(0,1fr\)\]\s+min-\[\d+px\]:\[grid-template-columns:(\d+)px/.exec(
        app,
      );
    expect(drawn).toBeTruthy();

    const rail = /^const RAIL = (\d+);/m.exec(paper)?.[1];
    const wide = /^const RAIL_WIDE = (\d+);/m.exec(paper)?.[1];

    expect(rail).toBe(drawn?.[1]);
    expect(wide).toBe(drawn?.[2]);
  });

  it("opens wide enough for the panel to stand beside the list", () => {
    const layout = readFileSync("src/layout.ts", "utf8");
    const needed = /@min-\[(\d+)px\]:pr-\[324px\]/.exec(layout)?.[1];
    expect(needed).toBeTruthy();

    const rail = 350;
    const gutter = 8;
    expect(main.width - rail - gutter).toBeGreaterThanOrEqual(Number(needed));
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
});
