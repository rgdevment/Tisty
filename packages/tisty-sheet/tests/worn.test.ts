import { describe, expect, it, vi } from "vitest";
import { PAGE_CSS, shell } from "../src/shell";

// Vitest hands CSS in as empty, so each sheet stands in as its own name.
vi.mock("../src/page.css?raw", () => ({ default: "/*page*/" }));
vi.mock("../src/tables.css?raw", () => ({ default: "/*tables*/" }));
vi.mock("../src/editor.css?raw", () => ({ default: "/*editor*/" }));

describe("what a page wears", () => {
  it("is the page's own CSS with its editor's and its tables', wherever the page goes", () => {
    expect(PAGE_CSS).toBe("/*page*//*editor*//*tables*/");
    expect(shell("<p>x</p>", false, "/*host*/")).toContain(
      "<style>/*page*//*editor*//*tables*//*host*/</style>",
    );
  });
});
