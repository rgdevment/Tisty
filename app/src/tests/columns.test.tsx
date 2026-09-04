import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot, Task } from "../core";

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
        return Promise.resolve(["2026-09-03"]);
      default:
        return Promise.resolve(held);
    }
  };
});

const cells = () => {
  const grid = document.querySelector('[class*="grid-cols-"]') as HTMLElement;
  const named = /grid-cols-\[([^\]]+)\]/.exec(grid.className);
  const tracks = named ? named[1].split("_").length : 0;
  return { grid, tracks, children: grid.children.length };
};

describe("the three columns of the window", () => {
  it("gives the board one cell, strip and all, so it never lands in a track of nought", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("write the report");

    await user.click(screen.getByRole("button", { name: /priorit/i }));
    await screen.findByRole("button", { name: /write the report/i });

    const before = cells();
    expect(before.children).toBeLessThanOrEqual(before.tracks);

    await user.click(screen.getByText("write the report"));
    await user.click(await screen.findByRole("button", { name: /^Complete$/ }));
    await screen.findByRole("region", { name: /did you do it/i });

    const after = cells();
    expect(after.children).toBeLessThanOrEqual(after.tracks);

    const board = after.grid.querySelector("section") as HTMLElement;
    expect(after.grid.children[0].contains(board)).toBe(true);
  });
});
