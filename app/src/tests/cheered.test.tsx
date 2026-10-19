import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot, Task } from "../core";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd, args ?? {});
  },
}));

const bus = vi.hoisted(() => ({
  heard: new Map<string, ((said: { payload: unknown }) => void)[]>(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (named: string, fn: (said: { payload: unknown }) => void) => {
    bus.heard.set(named, [...(bus.heard.get(named) ?? []), fn]);
    return Promise.resolve(() => {});
  },
  emit: () => Promise.resolve(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  save: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

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

const bread: Task = {
  id: "01A",
  title: "buy bread",
  status: "open",
  priority: "unset",
  order: "a0",
  steps: [],
  log: [],
  volume: {},
};

let tasks: Task[];

const shot = (view: { archive?: boolean } | undefined): Snapshot => ({
  tasks: tasks.filter((one) => (view?.archive ? one.status !== "open" : one.status === "open")),
  total: tasks.filter((one) => (view?.archive ? one.status !== "open" : one.status === "open"))
    .length,
  soonest: {},
  ahead: [],
  routines: [],
  lists: [],
  tags: [],
  refs: [],
  counts: {},
  locale: "en",
  agents: {},
});

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  bus.heard.clear();
  tasks = structuredClone([bread]);
  ipc.answer = (cmd, args) => {
    const held = tasks.find((one) => one.id === args.id);
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: [] });
      case "sync_state":
        return Promise.resolve({ asked: true });
      case "snapshot":
        return Promise.resolve(shot(args.view as { archive?: boolean } | undefined));
      case "settings":
        return Promise.resolve({
          quiet: [],
          holds: {},
          shares: false,
          onlySharedAbove: 0,
        });
      case "owed":
        return Promise.resolve([]);
      case "complete":
        if (held) held.status = "done";
        return Promise.resolve(held);
      case "reopen":
        if (held) held.status = "open";
        return Promise.resolve(held);
      default:
        return Promise.resolve(null);
    }
  };
});

describe("the thumbs up after closing a task", () => {
  it("shows up once a task is closed, and its undo opens the task again", async () => {
    render(<App />);
    await screen.findByText("buy bread");

    await userEvent.click(screen.getByRole("button", { name: /complete buy bread/i }));
    await userEvent.click(await screen.findByRole("button", { name: "Undo" }));

    await waitFor(() =>
      expect(ipc.calls.some((one) => one.cmd === "reopen" && one.args.id === "01A")).toBe(true),
    );
    await waitFor(() => expect(screen.queryByRole("button", { name: "Undo" })).toBeNull());
  });
});
