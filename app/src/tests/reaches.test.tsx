import { render, screen, waitFor } from "@testing-library/react";
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

const HOW_MANY = 1000;

const made = (at: number): Task =>
  ({
    id: `T${at}`,
    title: `una de tantas ${at}`,
    status: "open",
    priority: "unset",
    order: `a${at}`,
    steps: [],
    log: [],
    volume: {},
  }) as unknown as Task;

const all = Array.from({ length: HOW_MANY }, (_, at) => made(at));

let asked: number[];

const shot = (most: number): Snapshot =>
  ({
    tasks: all.slice(0, most),
    total: HOW_MANY,
    lists: [],
    tags: [],
    refs: [],
    counts: {},
    soonest: {},
    locale: "en",
  }) as unknown as Snapshot;

beforeEach(() => {
  localStorage.clear();
  asked = [];
  class Watching {
    private seen: () => void;
    constructor(seen: (entries: { isIntersecting: boolean }[]) => void) {
      this.seen = () => seen([{ isIntersecting: true }]);
    }
    observe() {
      this.seen();
    }
    disconnect() {}
    unobserve() {}
  }
  vi.stubGlobal("IntersectionObserver", Watching);

  ipc.answer = (cmd, args) => {
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: [] });
      case "sync_state":
        return Promise.resolve({ asked: true, loose: 0 });
      case "snapshot": {
        const view = (args.view ?? {}) as { most?: number };
        const most = view.most ?? HOW_MANY;
        asked.push(most);
        return Promise.resolve(shot(most));
      }
      case "owed":
        return Promise.resolve([]);
      default:
        return Promise.resolve(null);
    }
  };
});

describe("how far the column reaches", () => {
  it("asks for one more column when the end is reached, and then stops", async () => {
    render(<App />);
    await screen.findByText("una de tantas 0");

    await waitFor(() => expect(asked.length).toBeGreaterThan(1));
    await new Promise((done) => setTimeout(done, 50));

    expect(asked).toEqual([200, 400]);
  });

  it("never asks the next view for the reach the last one had grown to", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("una de tantas 0");
    await waitFor(() => expect(asked).toContain(400));

    const upto = asked.length;
    await user.click(screen.getByRole("button", { name: /priorit/i }));
    await waitFor(() => expect(asked.length).toBeGreaterThan(upto));
    await new Promise((done) => setTimeout(done, 50));

    expect(asked.slice(upto)).toEqual([200]);
  });
});
