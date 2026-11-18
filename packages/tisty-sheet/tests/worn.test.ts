import { describe, expect, it, vi } from "vitest";
import { PAGE_CSS, shell } from "../src/shell";

// Vitest hands CSS in as empty, so each sheet stands in as its own name.
vi.mock("../src/page.css?raw", () => ({ default: "/*page*/" }));
vi.mock("../src/callouts.css?raw", () => ({ default: "/*callouts*/" }));
vi.mock("../src/code.css?raw", () => ({ default: "/*code*/" }));
vi.mock("../src/diagrams.css?raw", () => ({ default: "/*diagrams*/" }));
vi.mock("../src/tables.css?raw", () => ({ default: "/*tables*/" }));
vi.mock("../src/editor.css?raw", () => ({ default: "/*editor*/" }));

describe("what a page wears", () => {
  it("is the page's own CSS, then its blocks' where they stood in it, its editor's and its tables', wherever the page goes", () => {
    const worn = "/*page*//*callouts*//*code*//*diagrams*//*editor*//*tables*/";
    expect(PAGE_CSS).toBe(worn);
    expect(shell("<p>x</p>", false, "/*host*/")).toContain(`<style>${worn}/*host*/</style>`);
  });
});
