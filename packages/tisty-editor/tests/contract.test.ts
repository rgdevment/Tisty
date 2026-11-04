import { CONTRACTS } from "@tisty/contracts";
import { describe, expect, it } from "vitest";
import { source } from "./source";

type Blocks = Record<string, Record<string, string | number>>;
type Style = {
  values: {
    colors: { light: Record<string, string>; dark: Record<string, string> };
    body: { size: string; line_height: number };
    headings: Record<string, { size: string }>;
    code: { size: string; line_height: number };
    table: { size: string; margin: string; cell_padding: string };
    sheet: { width: string };
    blocks: Blocks;
    callouts: {
      hues: Record<string, string>;
      tint_percent: number;
      padding: string;
      border_width: string;
    };
    code_colors: Record<string, { color: string; italic?: boolean }>;
    icon: { size: string };
    text_font: string[];
    code_font: string[];
  };
};
type Sheet = {
  values: {
    millimetres: Record<string, [number, number]>;
    margin_millimetres: { x: number; y: number };
    offered: string[];
    long_page_width_pixels: number;
  };
};

const style = (CONTRACTS.style as Style).values;
const sheet = (CONTRACTS.sheet as Sheet).values;
const page = source("page.css");
const tables = source("tables.css");
const tokens = source("tokens.css");
const paper = source("paper.ts");

// A comma inside :is() or :not() does not end a selector.
const selectorsIn = (said: string) => said.split(/,(?![^(]*\))/).map((one) => one.trim());

const each = (css: string) =>
  [...css.replace(/\/\*[\s\S]*?\*\//g, "").matchAll(/([^{}]+)\{([^{}]*)\}/g)].map(
    ([, selector, body]) => ({
      selectors: selectorsIn(selector),
      body,
    }),
  );

// The value a selector is given, as a reader of this stylesheet would find it when alone.
const said = (css: string, selector: string, property: string): string | undefined =>
  each(css)
    .filter((one) => one.selectors.includes(selector))
    .map((one) => new RegExp(`(?:^|[;\\s])${property}:\\s*([^;]+);`).exec(one.body)?.[1])
    .find(Boolean);

const colorsOf = (css: string) =>
  Object.fromEntries(
    [...css.matchAll(/--tisty-([a-z-]+):\s*([^;]+);/g)].map(([, name, value]) => [name, value]),
  );

describe("what the contract says a sheet looks like is what Tisty's own sheet wears", () => {
  it("its colours, in each theme", () => {
    const [light, dark] = tokens.split('[data-theme="dark"]');
    expect(colorsOf(light)).toEqual(style.colors.light);
    expect(colorsOf(dark)).toEqual(style.colors.dark);
  });

  it("its letters", () => {
    const body = said(page, "body", "font")?.replace(/\s+/g, " ");
    expect(body).toContain(`${style.body.size} / ${style.body.line_height}`);
    for (const one of style.text_font) expect(body).toContain(one);
    const mono = page.match(/font-family: (ui-monospace[^;]+);/)?.[1] ?? "";
    for (const one of style.code_font) expect(mono).toContain(one);
    expect(said(page, "code", "font-size")).toBe(style.code.size);
  });

  it("its headings", () => {
    for (const [tag, one] of Object.entries(style.headings))
      expect(said(page, tag, "font-size")).toBe(one.size);
  });

  it("the margins that decide where a sheet is cut", () => {
    const { blocks } = style;
    for (const tag of ["h1", "h2", "h3", "h4", "p"])
      expect(said(page, tag, "margin")).toBe(blocks[tag].margin);
    expect(said(page, "ul", "margin")).toBe(blocks.list.margin);
    expect(said(page, "ul", "padding-left")).toBe(blocks.list.padding_left);
    expect(said(page, "li > p", "margin")).toBe(blocks.list_paragraph.margin);
    expect(said(page, "blockquote", "margin")).toBe(blocks.quote.margin);
    expect(said(page, "blockquote", "padding-left")).toBe(blocks.quote.padding_left);
    expect(said(page, "figure", "margin")).toBe(blocks.figure.margin);
    expect(said(page, "pre", "padding")).toBe(blocks.pre.padding);
    expect(said(page, "pre", "line-height")).toBe(String(blocks.pre.line_height));
    expect(said(page, "details", "margin")).toBe(blocks.details.margin);
    for (const tag of ["h5", "h6"]) expect(said(page, tag, "margin")).toBe(blocks.h4.margin);
    expect(said(page, "img", "margin")).toBe(blocks.picture.margin);
    expect(said(page, "img", "padding")).toBe(blocks.picture.padding);
    expect(said(page, "img", "max-height")).toBe(blocks.picture.max_height);
    expect(said(page, "hr", "margin")).toBe(blocks.rule.margin);
    expect(said(page, "div.math", "margin")).toBe(blocks.formula.margin);
    expect(said(page, "div.math", "padding")).toBe(blocks.formula.padding);
    expect(said(page, "figure.code figcaption", "padding")).toBe(blocks.caption.padding);
    expect(said(page, ".footnote", "margin")).toBe(blocks.note.margin);
    expect(said(page, ".refdef", "margin")).toBe(blocks.definition.margin);
    expect(said(page, ".front", "display")).toBe("none");
    expect(said(page, "figure.code", "border-radius")).toBe(blocks.code_figure.radius);
    expect(said(page, "details", "border-radius")).toBe(blocks.details.radius);
    expect(said(page, "summary", "padding")).toBe(blocks.details.summary_padding);
    expect(said(page, ".footnote", "gap")).toBe(blocks.note.gap);
    expect(said(page, ".check", "left")).toBe(blocks.check.left);
    expect(said(page, ".check", "top")).toBe(blocks.check.top);
  });

  it("its tables", () => {
    expect(said(tables, "table", "font-size")).toBe(style.table.size);
    expect(said(tables, "table", "margin")).toBe(style.table.margin);
    expect(tables).toContain(`padding: ${style.table.cell_padding}`);
  });

  it("the colours of its notices", () => {
    expect(said(page, "blockquote.callout", "padding")).toBe(style.callouts.padding);
    expect(said(page, "blockquote.callout", "--said")).toBe(
      `var(--tisty-hue-${style.callouts.hues.note})`,
    );
    for (const kind of ["tip", "important", "warning", "caution"]) {
      expect(said(page, `blockquote[data-callout="${kind}"]`, "--said")).toBe(
        `var(--tisty-hue-${style.callouts.hues[kind]})`,
      );
    }
    expect(said(page, "blockquote.callout", "background")).toContain(
      `${style.callouts.tint_percent}%`,
    );
  });

  it("the colours of its code", () => {
    for (const [name, one] of Object.entries(style.code_colors)) {
      expect(said(page, `.hljs-${name}`, "color"), name).toBe(`var(--tisty-${one.color})`);
    }
  });

  it("its icons", () => {
    expect(said(page, ".ico svg", "width")).toBe(style.icon.size);
  });

  it("its paper", () => {
    for (const [name, [wide, tall]] of Object.entries(sheet.millimetres)) {
      expect(paper).toContain(`${name}: [${wide}, ${tall}]`);
    }
    const { x, y } = sheet.margin_millimetres;
    expect(paper).toContain(`MARGIN_MM = { x: ${x}, y: ${y} }`);
    expect(paper).toContain(
      `LEAVES: Paper[] = [${sheet.offered.map((one) => `"${one}"`).join(", ")}]`,
    );
    expect(page).toContain(`--wide, ${sheet.long_page_width_pixels}px`);
    expect(style.sheet.width).toBe(`${sheet.long_page_width_pixels}px`);
  });
});
