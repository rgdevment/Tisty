import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fireEvent, render, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Pick from "../ui/Pick";

const said = readFileSync(resolve(process.cwd(), "../crates/tisty-core/src/model/icon.rs"), "utf8");

const named = (): string[] => {
  const body = said.slice(said.indexOf("ICONS"), said.indexOf("];"));
  return [...body.matchAll(/"([a-z-]+)"/g)].map((found) => found[1]);
};

const cuts = (): [string, number][] => {
  const body = said.slice(said.indexOf("pub const FAMILIES"));
  const rows = body.slice(0, body.indexOf("];")).matchAll(/\("([a-z]+)", (\d+)\)/g);
  return [...rows].map((found) => [found[1], Number(found[2])] as [string, number]);
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "icons" ? named() : cuts()),
}));

/// The window measures the grid after it is on screen, so the picker is drawn twice.
let wide = 0;
const watching: (() => void)[] = [];

class Watcher {
  constructor(private tell: () => void) {}
  observe() {
    watching.push(this.tell);
  }
  disconnect() {}
}

const opened = async () => {
  vi.stubGlobal("ResizeObserver", Watcher);
  const { container } = render(<Pick onIcon={vi.fn()} />);
  const box = container.querySelector("fieldset") as HTMLFieldSetElement;
  wide = 377;
  for (const tell of watching) tell();
  await waitFor(() => expect(box.querySelectorAll("button").length).toBeLessThan(200));
  return box;
};

Object.defineProperty(HTMLFieldSetElement.prototype, "clientWidth", {
  configurable: true,
  get: () => wide,
});
Object.defineProperty(HTMLFieldSetElement.prototype, "clientHeight", {
  configurable: true,
  get: () => (wide ? 208 : 0),
});

const slide = async (box: HTMLFieldSetElement, down: number) => {
  fireEvent.scroll(box, { target: { scrollTop: down } });
  await waitFor(() => expect(box.querySelectorAll("button").length).toBeGreaterThan(0));
};

describe("a picker being scrolled", () => {
  it("lets go of the family it has left behind", async () => {
    const box = await opened();
    expect(within(box).getByText("Home")).toBeTruthy();

    await slide(box, 3000);

    expect(within(box).queryByText("Home")).toBeNull();
    expect(box.querySelectorAll("p").length).toBeLessThan(3);
  });

  it("keeps drawing icons the whole way down", async () => {
    const box = await opened();

    for (const down of [600, 1400, 3000, 5000]) {
      await slide(box, down);
      expect(box.querySelectorAll("svg").length).toBeGreaterThan(50);
    }
  });

  it("gives no row the name of another, however a family and an icon are called", async () => {
    const cried = vi.spyOn(console, "error").mockImplementation(() => {});
    const box = await opened();
    for (const down of [200, 600, 1400, 3000]) await slide(box, down);

    expect(cried.mock.calls.filter(([first]) => String(first).includes("same key"))).toEqual([]);
    cried.mockRestore();
  });
});
