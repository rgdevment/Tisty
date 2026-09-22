import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot, Task } from "../core";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  due: false,
  agents: false,
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
  ipc.due = false;
  ipc.agents = false;
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
        return Promise.resolve({ asked: true, backsUp: true, loose: 0 });
      case "snapshot":
        return Promise.resolve(shot(args.view as { archive?: boolean } | undefined));
      case "owed":
        return Promise.resolve([]);
      case "star_due":
        return Promise.resolve(ipc.due);
      case "door_due":
        return Promise.resolve(ipc.agents);
      case "complete":
        if (held) held.status = "done";
        return Promise.resolve(held);
      default:
        return Promise.resolve(held);
    }
  };
});

const started = async () => {
  render(<App />);
  await screen.findByText("buy bread");
};

const complete = async () =>
  await userEvent.click(screen.getByRole("button", { name: /complete buy bread/i }));

const card = () => screen.queryByRole("status", { name: /support tisty/i });

const door = () => screen.queryByRole("status", { name: /your assistants/i });

describe("when the card is allowed on screen", () => {
  it("stays away while the door is shut, and is asked about all the same", async () => {
    await started();
    await complete();

    await waitFor(() => expect(ipc.calls.some((one) => one.cmd === "star_due")).toBe(true));
    expect(card()).toBeNull();
  });

  it("arrives once the door is open, after a task is closed", async () => {
    ipc.due = true;
    await started();
    await complete();

    await waitFor(() => expect(card()).toBeTruthy());
  });

  it("asks nothing when nothing was closed", async () => {
    ipc.due = true;
    await started();

    expect(ipc.calls.some((one) => one.cmd === "star_due")).toBe(false);
    expect(card()).toBeNull();
  });
});

describe("what takes the card away without answering it", () => {
  it("goes when the window withdraws to the tray", async () => {
    ipc.due = true;
    await started();
    await complete();
    await waitFor(() => expect(card()).toBeTruthy());

    await act(async () => {
      for (const fn of bus.heard.get("withdrawn") ?? []) fn({ payload: null });
    });

    expect(card()).toBeNull();
    expect(ipc.calls.some((one) => one.cmd === "star_done")).toBe(false);
  });

  it("goes when the person moves to another screen", async () => {
    ipc.due = true;
    await started();
    await complete();
    await waitFor(() => expect(card()).toBeTruthy());

    await userEvent.click(screen.getByRole("button", { name: /^archive$/i }));

    await waitFor(() => expect(card()).toBeNull());
    expect(ipc.calls.some((one) => one.cmd === "star_done")).toBe(false);
  });
});

describe("the card that offers the assistant a door", () => {
  it("waits for nothing to be finished, unlike the star", async () => {
    ipc.agents = true;
    await started();

    await waitFor(() => expect(door()).toBeTruthy());
  });

  it("stays away on a machine with no assistant on it", async () => {
    await started();

    await waitFor(() => expect(ipc.calls.some((one) => one.cmd === "door_due")).toBe(true));
    expect(door()).toBeNull();
  });

  it("goes first when the star falls due at the same moment", async () => {
    ipc.due = true;
    ipc.agents = true;
    await started();
    await complete();

    await waitFor(() => expect(door()).toBeTruthy());
    expect(card()).toBeNull();
  });

  it("walks to the assistants, not to the settings at large", async () => {
    ipc.agents = true;
    await started();
    await waitFor(() => expect(door()).toBeTruthy());

    await userEvent.click(screen.getByRole("button", { name: /open settings/i }));

    await waitFor(() => expect(screen.getByRole("tab", { name: /assistants/i })).toBeTruthy());
    expect(screen.getByRole("tab", { name: /assistants/i }).getAttribute("aria-selected")).toBe(
      "true",
    );
  });
});
