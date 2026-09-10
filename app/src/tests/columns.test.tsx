import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot, Task } from "../core";

/// Yesterday, not a date written down: a day owed has to stay owed whenever this is run.
const yesterday = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString().slice(0, 10);

const ipc = vi.hoisted(() => ({
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => ipc.answer(cmd, args ?? {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  save: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
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

const report: Task = {
  id: "01A",
  title: "write the report",
  status: "open",
  priority: "do",
  order: "a0",
  steps: [],
  log: [],
  volume: {},
} as unknown as Task;

let tasks: Task[];

const shot = (): Snapshot =>
  ({
    tasks: tasks.filter((one) => one.status === "open"),
    lists: [],
    tags: [],
    refs: [],
    counts: {},
    locale: "en",
  }) as unknown as Snapshot;

beforeEach(() => {
  localStorage.clear();
  tasks = structuredClone([report]);
  ipc.answer = (cmd, args) => {
    const held = tasks.find((one) => one.id === args.id);
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: [] });
      case "sync_state":
        return Promise.resolve({ asked: true, backsUp: true, loose: 0 });
      case "snapshot":
        return Promise.resolve(shot());
      case "task_story":
        return Promise.resolve({ id: String(args.id ?? ""), pages: [] });
      case "read":
        return Promise.resolve({ title: String(args.text ?? ""), tags: [], spans: [], offers: [] });
      case "owed":
        return Promise.resolve([yesterday]);
      default:
        return Promise.resolve(held);
    }
  };
});

Object.defineProperty(window, "innerWidth", { configurable: true, writable: true, value: 1440 });

const lane = () => document.querySelector('[class*="transition-[padding]"]') as HTMLElement;

describe("the room the desk keeps for the lanes beside it", () => {
  beforeEach(() => {
    // The board takes the pointer when a card is pressed, and jsdom has no such thing.
    Element.prototype.setPointerCapture = () => {};
    Element.prototype.releasePointerCapture = () => {};
  });

  it("holds the board itself, so nothing of it hides under a lane", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("write the report");

    await user.click(screen.getByRole("button", { name: /priorit/i }));
    await screen.findByRole("button", { name: /write the report/i });

    const board = document.querySelector("section") as HTMLElement;
    expect(lane().contains(board)).toBe(true);
  });

  it("keeps room for the day on its own, and for the task beside it", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("write the report");

    expect(lane().className).toContain("pr-[324px]");

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    expect(lane().className).toContain("pr-[404px]");
    expect(lane().className).toContain("pr-[716px]");
  });

  it("keeps no room at all in the spread, where seven days want the whole width", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("write the report");

    await user.click(screen.getByRole("button", { name: /spread/i }));
    await screen.findByRole("button", { name: /the week after/i });

    expect(lane().className).not.toContain("pr-[");

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    expect(lane().className).not.toContain("pr-[");
  });
});
