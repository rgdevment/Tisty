import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { known, markup } from "../glyphs";
import Pick from "../ui/Pick";

const named = (): string[] => {
  const at = resolve(process.cwd(), "../crates/tisty-core/src/model/icon.rs");
  const said = readFileSync(at, "utf8");
  const body = said.slice(said.indexOf("ICONS"), said.indexOf("];"));
  return [...body.matchAll(/"([a-z-]+)"/g)].map((found) => found[1]);
};

const drawn = (): string[] => {
  const said = readFileSync(resolve(process.cwd(), "src/glyphs.ts"), "utf8");
  const rows = said.matchAll(/^ {2}("[^"]+"|[a-z][a-z0-9]*): \d+,$/gm);
  return [...rows].map((found) => found[1].replace(/"/g, ""));
};

/// The window draws these itself — its menus and its tree — so the core never offers them.
const OURS = [
  "aligncenter",
  "alignleft",
  "alignright",
  "bullets",
  "checks",
  "doc",
  "emoji",
  "grid",
  "heading1",
  "heading2",
  "highlight",
  "numbers",
  "pagebreak",
  "picture",
  "plus",
  "quote",
  "rows",
  "rule",
  "table",
  "tag",
  "today",
];

const all = named();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "icons" ? named() : null),
}));

describe("the catalogue the core hands over", () => {
  it("carries a drawing for every name, so none is offered and then withheld", () => {
    expect(all.length).toBeGreaterThan(1300);
    expect(all.filter((key) => !known(key))).toEqual([]);
  });

  it("holds no drawing the core cannot name, beyond the window's own", () => {
    expect(drawn().filter((key) => !all.includes(key))).toEqual(OURS);
  });
});

describe("the table of drawings, which is held apart from its keys", () => {
  it("still hangs each key on its own shape, however the table was reordered", () => {
    expect(markup("home")).toContain('d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"');
    expect(markup("wine")).toContain(
      'd="M12 15a5 5 0 0 0 5-5c0-2-.5-4-2-8H9c-1.5 4-2 6-2 8a5 5 0 0 0 5 5Z"',
    );
    expect(markup("zoom")).toContain('<line x1="8" x2="14" y1="11" y2="11" />');
  });
});

describe("a catalogue too long to draw at once", () => {
  it("lays out the rows in view rather than every icon it has", async () => {
    Object.defineProperty(HTMLFieldSetElement.prototype, "clientWidth", { value: 200 });
    Object.defineProperty(HTMLFieldSetElement.prototype, "clientHeight", { value: 200 });

    render(<Pick onIcon={vi.fn()} />);
    await waitFor(() => screen.getByRole("button", { name: "home" }));

    expect(screen.getAllByRole("button").length).toBeLessThan(all.length / 10);
  });
});
