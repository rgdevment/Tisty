import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot, Task } from "../core";

const ipc = vi.hoisted(() => ({
  parts: (): Promise<unknown> => Promise.resolve([]),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: true, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: [] });
      case "sync_state":
        return Promise.resolve({ asked: true });
      case "snapshot":
        return Promise.resolve(shot());
      case "parts_of":
        return ipc.parts();
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ onDragDropEvent: () => Promise.resolve(() => {}) }),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
  }),
}));

const task = (id: string, title: string, more: Partial<Task> = {}): Task => ({
  id,
  title,
  status: "open",
  priority: "unset",
  order: id,
  steps: [],
  log: [],
  volume: {},
  ...more,
});

const shot = (): Snapshot => ({
  tasks: [task("01A3", "A3 · Migración de Sodimac Uruguay")],
  total: 1,
  soonest: {},
  ahead: [],
  routines: [],
  lists: [],
  tags: [],
  refs: [],
  counts: {},
  locale: "en",
  agents: {},
  wholes: { "01A3": { title: "A3 · Migración de Sodimac Uruguay", open: 1, closed: 0 } },
});

beforeEach(() => {
  localStorage.clear();
  ipc.parts = () => Promise.resolve([task("01P1", "Inventario", { part_of: "01A3" })]);
});

describe("a part the view leaves out, opened from the list", () => {
  it("opens its detail although the view does not hold it", async () => {
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: /0 of 1 parts closed/ }));
    await userEvent.click(await screen.findByText("Inventario"));

    const titled = await screen.findByRole("textbox", { name: "Title" });
    expect((titled as HTMLInputElement).value ?? titled.textContent).toContain("Inventario");
  });

  it("says so when its parts cannot be read", async () => {
    ipc.parts = () => Promise.reject(new Error("the history is locked"));
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: /0 of 1 parts closed/ }));

    expect(await screen.findByText(/the history is locked/)).not.toBeNull();
  });
});
