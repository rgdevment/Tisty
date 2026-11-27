import { act, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot } from "../core";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  came: "same",
  mark: "m1",
  body: "# Largo\n\nUno dos tres.\n",
}));

const drawn = vi.hoisted(() => ({ sidebar: 0 }));

vi.mock("../ui/Sidebar", async (real) => {
  const { default: Sidebar } = await real<typeof import("../ui/Sidebar")>();
  return {
    default: (props: Parameters<typeof Sidebar>[0]) => {
      drawn.sidebar += 1;
      return Sidebar(props);
    },
  };
});

const bus = vi.hoisted(() => ({
  heard: new Map<string, ((said: { payload: unknown }) => void)[]>(),
}));

const DOC = {
  id: "01D",
  file: "a3f1-0001",
  title: "Largo",
  folder: null,
  archived: false,
  away: false,
};

const TASKS = Array.from({ length: 3 }, (_, at) => ({
  id: `01A${at}`,
  title: at === 0 ? "write the report" : `task number ${at}`,
  status: "open" as const,
  priority: "unset" as const,
  order: `a${String(at).padStart(4, "0")}`,
  steps: Array.from({ length: 3 }, (_, step) => ({
    id: `01S${at}-${step}`,
    text: `step ${step}`,
    done: false,
    order: `a${step}`,
  })),
  log: [],
  volume: { steps: 3, steps_done: 0 },
}));
const DOCS = Array.from({ length: 3 }, (_, at) =>
  at === 0 ? DOC : { ...DOC, id: `01D${at}`, file: `b${at}-0001`, title: `Doc ${at}` },
);

const shot = (): Snapshot => ({
  tasks: structuredClone(TASKS),
  total: TASKS.length,
  soonest: {},
  ahead: [],
  routines: [],
  lists: [],
  tags: [],
  counts: {},
  locale: "en",
  agents: {},
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: structuredClone(DOCS) });
      case "sync_state":
        return Promise.resolve({ asked: true, chosen: "G:/My Drive/tisty" });
      case "folder_astir":
        return Promise.resolve(ipc.mark);
      case "sync_now":
        return Promise.resolve({ carried: ipc.came, undecided: [], unreadable: [] });
      case "snapshot":
        return Promise.resolve(shot());
      case "doc_opened":
        return Promise.resolve({ body: ipc.body, print: "p1" });
      case "doc_read":
        return Promise.resolve(ipc.body);
      default:
        return Promise.resolve(null);
    }
  },
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

const told = (named: string, payload?: unknown) => {
  for (const fn of bus.heard.get(named) ?? []) fn({ payload });
};

const many = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd).length;

beforeEach(() => {
  localStorage.clear();
  ipc.came = "same";
  ipc.mark = "m1";
  bus.heard.clear();
  ipc.calls = [];
});

const opened = async () => {
  render(<App />);
  await screen.findByText("write the report");
  await act(async () => told("showdoc", DOC.file));
  await waitFor(() => expect(many("doc_opened")).toBeGreaterThan(0));
  await act(() => vi.advanceTimersByTimeAsync(2000));
  ipc.calls = [];
  drawn.sidebar = 0;
};

describe("a sync round landing over an open document", () => {
  it("asks the core once for what the round told in one breath", async () => {
    await opened();

    await act(async () => told("carried", "log"));
    await act(() => vi.advanceTimersByTimeAsync(100));
    await act(async () => told("carried", "papers"));
    await act(() => vi.advanceTimersByTimeAsync(1000));

    expect(many("snapshot")).toBe(1);
    expect(many("docs")).toBe(1);
    expect(many("doc_opened")).toBe(1);
  });

  it("asks again for what a later telling brings", async () => {
    await opened();

    await act(async () => told("carried", "log"));
    await act(() => vi.advanceTimersByTimeAsync(2000));
    await act(async () => told("stirred"));
    await act(() => vi.advanceTimersByTimeAsync(2000));

    expect(many("snapshot")).toBe(2);
    expect(many("docs")).toBe(1);
    expect(many("doc_opened")).toBe(2);
  });

  it("draws the open document again, not the sidebar, when the round brought nothing new", async () => {
    await opened();

    await act(async () => told("carried", "log"));
    await act(() => vi.advanceTimersByTimeAsync(1000));

    expect(many("doc_opened")).toBe(1);
    expect(drawn.sidebar).toBe(0);
  });
});

describe("a sync round that brought words, with the telling of the same breath", () => {
  const roundBringing = async () => {
    ipc.came = "came";
    ipc.mark = "m2";
    await act(async () => told("told"));
  };

  it("asks the core once for what the round and a telling 50 ms later both brought", async () => {
    await opened();

    await roundBringing();
    await act(() => vi.advanceTimersByTimeAsync(50));
    await act(async () => told("carried", "log"));
    await act(() => vi.advanceTimersByTimeAsync(1000));

    expect(many("sync_now")).toBeGreaterThan(0);
    expect(many("snapshot")).toBe(1);
    expect(many("doc_opened")).toBe(1);
  });

  it("asks the core once for a note jotted and a paper carried 50 ms apart", async () => {
    await opened();

    await act(async () => told("jotted"));
    await act(() => vi.advanceTimersByTimeAsync(50));
    await act(async () => told("carried", "papers"));
    await act(() => vi.advanceTimersByTimeAsync(1000));

    expect(many("docs")).toBe(1);
    expect(many("doc_opened")).toBe(1);
  });
});
