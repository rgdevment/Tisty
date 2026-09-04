import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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
  const { container } = render(<Pick onIcon={vi.fn()} opens="icons" />);
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

const window = (box: HTMLFieldSetElement) => {
  const held = box.querySelector(".relative") as HTMLElement;
  const moved = box.querySelector(".absolute") as HTMLElement;
  const total = Number.parseFloat(held.style.height);
  const first = Number.parseFloat(moved.style.transform.replace(/[^\d.-]/g, ""));
  return { total, first };
};

describe("a window scrolled past the end of what it holds", () => {
  it("keeps the grid full rather than sliding an empty page into view", async () => {
    const box = await opened();

    fireEvent.scroll(box, { target: { scrollTop: 100000 } });
    await waitFor(() => expect(box.querySelectorAll("button").length).toBeGreaterThan(0));

    const { total, first } = window(box);
    expect(first).toBeGreaterThanOrEqual(0);
    expect(first).toBeLessThanOrEqual(total);
  });

  it("never asks for a negative slice of rows", async () => {
    const box = await opened();

    fireEvent.scroll(box, { target: { scrollTop: -5000 } });
    await waitFor(() => expect(box.querySelectorAll("button").length).toBeGreaterThan(0));

    const { first } = window(box);
    expect(first).toBeGreaterThanOrEqual(0);
  });
});

describe("a picker whose family narrows and then widens back out", () => {
  it("starts the next family at the top, not wherever the last one left off", async () => {
    const box = await opened();
    await userEvent.click(screen.getByRole("button", { name: "Kitchen" }));
    fireEvent.scroll(box, { target: { scrollTop: 400 } });
    await waitFor(() => expect(box.scrollTop).toBe(400));

    await userEvent.click(screen.getByRole("button", { name: "All" }));

    expect(box.scrollTop).toBe(0);
    expect(within(box).getByText("Home")).toBeTruthy();
  });
});

describe("a picker whose box is resized", () => {
  it("relays the icons out under the new width without losing or doubling any of them", async () => {
    const box = await opened();
    const before = box.querySelectorAll("svg").length;

    wide = 600;
    for (const tell of watching) tell();
    await waitFor(() => expect(box.querySelectorAll("svg").length).not.toBe(before));

    const cried = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(box.querySelectorAll("svg").length).toBeGreaterThan(0);
    expect(cried.mock.calls.filter(([first]) => String(first).includes("same key"))).toEqual([]);
    cried.mockRestore();
  });
});
