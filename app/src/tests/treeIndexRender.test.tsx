import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Filed, Papers } from "../core";
import Tree from "../ui/Tree";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve([]),
}));

const doc = (
  id: string,
  title: string,
  folder: string | null,
  over: Partial<Filed> = {},
): Filed => ({
  id,
  file: id,
  title,
  folder,
  archived: false,
  away: false,
  ...over,
});

const papers: Papers = {
  folders: [
    { id: "F1", name: "work", parent: null, icon: null, holds: 3, archived: false, away: false },
  ],
  docs: [
    doc("D3", "Gamma", "F1"),
    doc("D1", "Alpha", "F1"),
    doc("D5", "Loose two", null),
    doc("D2", "Beta", "F1"),
    doc("D4", "Loose one", null),
    doc("P2", "Page B", "F1", { pageOf: "D1" }),
    doc("P1", "Page A", "F1", { pageOf: "D1" }),
    doc("D6", "Old", "F1", { archived: true, away: true }),
  ],
};

const order = (titles: string[]) => {
  const at = (title: string) =>
    Array.from(document.body.querySelectorAll("button")).findIndex(
      (one) => one.getAttribute("aria-label") === title || one.textContent?.startsWith(title),
    );
  return titles.map(at);
};

describe("the tree reading its index", () => {
  const show = () => {
    render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} />);
    for (let round = 0; round < 8; round += 1) {
      const shut = screen.queryAllByRole("button", { name: /^Open / });
      const shelf = screen
        .queryAllByRole("button", { name: "Archived" })
        .filter((one) => one.getAttribute("aria-expanded") === "false");
      for (const one of [...shut, ...shelf]) fireEvent.click(one);
    }
  };

  it("shows a folder's documents in list order, pages left out and the archived one after them", () => {
    show();
    const places = order(["Gamma", "Alpha", "Beta"]);
    expect(places.every((one) => one >= 0)).toBe(true);
    expect([...places].sort((a, b) => a - b)).toEqual(places);
    expect(screen.queryByRole("button", { name: "Page A" })).toBeNull();
    expect(order(["Old"])[0]).toBeGreaterThan(order(["Beta"])[0]);
  });

  it("shows the loose documents in list order", () => {
    show();
    const [two, one] = order(["Loose two", "Loose one"]);
    expect(two).toBeGreaterThanOrEqual(0);
    expect(two).toBeLessThan(one);
  });

  it("counts the pages and lists them in order once spread", () => {
    show();
    expect(screen.getByRole("button", { name: "Alpha" }).textContent).toContain("2 pages");
    fireEvent.click(screen.getByRole("button", { name: "Show the pages of Alpha" }));
    const [b, a] = order(["Page B", "Page A"]);
    expect(b).toBeGreaterThanOrEqual(0);
    expect(b).toBeLessThan(a);
  });

  it("keeps the archived document on the shelf", () => {
    show();
    expect(screen.getByRole("button", { name: /^Old/ })).toBeTruthy();
  });
});
