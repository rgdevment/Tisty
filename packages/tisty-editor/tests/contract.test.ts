import { CONTRACTS } from "@tisty/contracts";
import { describe, expect, it } from "vitest";
import { fields } from "../src/front";
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

type FrontMatter = {
  values: {
    block_bytes_at_most: number;
    properties_at_most: number;
    value_characters_at_most: number;
  };
};

const style = (CONTRACTS.style as Style).values;
const sheet = (CONTRACTS.sheet as Sheet).values;
const frontMatter = (CONTRACTS.front_matter as FrontMatter).values;
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

  describe("a diagram that arrives drawn twice", () => {
    const drawings = each(page)
      .filter((one) => one.selectors.some((selector) => selector.includes("> svg[data-theme")))
      .map((one) => `${one.selectors.join(",")}{${one.body}}`)
      .join("\n");

    const shownIn = (theme: string | null) => {
      if (theme) document.documentElement.setAttribute("data-theme", theme);
      else document.documentElement.removeAttribute("data-theme");
      const style = document.createElement("style");
      style.textContent = drawings;
      document.head.append(style);
      document.body.innerHTML =
        '<figure class="diagram"><svg data-theme="light"></svg><svg data-theme="dark"></svg></figure>';
      const seen = [...document.querySelectorAll("svg")]
        .filter((one) => getComputedStyle(one).display !== "none")
        .map((one) => one.getAttribute("data-theme"));
      style.remove();
      document.body.innerHTML = "";
      document.documentElement.removeAttribute("data-theme");
      return seen;
    };

    it("shows only the one of the reader's theme", () => {
      expect(shownIn("light")).toEqual(["light"]);
      expect(shownIn("dark")).toEqual(["dark"]);
      expect(shownIn(null)).toEqual(["light"]);
    });

    it("draws each at most at its width and at most at the width of the sheet", () => {
      expect(said(page, "figure.diagram > svg[data-theme]", "max-width")).toBe("100%");
      expect(said(page, "figure.diagram > svg[data-theme]", "width")).toBe("auto");
      expect(said(page, "figure.diagram svg", "margin")).toBe("0 auto");
    });

    it("wears no frame of a diagram still waiting to be drawn", () => {
      expect(page).toContain("figure.diagram:not(.drawn, :has(> svg[data-theme]))");
    });
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

describe("the window reads a document's properties as far as the contract says", () => {
  it("up to its bytes", () => {
    const frame = "---\ntitle: a\n#\n---\n".length;
    const sized = (bytes: number) => `---\ntitle: a\n#${"x".repeat(bytes - frame)}\n---\n`;
    const atMost = frontMatter.block_bytes_at_most;
    expect(sized(atMost)).toHaveLength(atMost);
    expect(fields(sized(atMost))).toHaveLength(1);
    expect(fields(sized(atMost + 1))).toEqual([]);
    expect(fields(sized(atMost + 1).trimEnd())).toHaveLength(1);
    expect(fields(sized(atMost + 2).trimEnd())).toEqual([]);
  });

  it("up to its properties", () => {
    const keyed = (keys: number) => {
      const others = Array.from({ length: keys - 1 }, (_, at) => `k${at}: v\n`).join("");
      return `---\ntitle: a\n${others}---\n`;
    };
    const atMost = frontMatter.properties_at_most;
    expect(fields(keyed(atMost))).toHaveLength(atMost);
    expect(fields(keyed(atMost + 1))).toEqual([]);
  });

  it("up to the characters of a value", () => {
    // One emoji is one character but two units of a string, so counting units reads less than the contract says.
    const said = (characters: number) => `---\ntitle: 😀${"x".repeat(characters - 1)}\n---\n`;
    const atMost = frontMatter.value_characters_at_most;
    expect(fields(said(atMost))).toHaveLength(1);
    expect(fields(said(atMost + 1))).toEqual([]);
  });
});
