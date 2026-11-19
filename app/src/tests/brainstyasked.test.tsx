import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Neuron } from "../brainsty";
import type { Snapshot, Task } from "../core";
import { t } from "../locales";
import { openedHere } from "../ui/BrainstyAsked";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  task: (): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return answer(cmd);
  },
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
}));

const heard = vi.hoisted(() => ({
  on: new Map<string, (event: { payload: unknown }) => void>(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (event: { payload: unknown }) => void) => {
    heard.on.set(name, handler);
    return Promise.resolve(() => heard.on.delete(name));
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

const shot: Snapshot = {
  tasks: [bread],
  total: 1,
  soonest: {},
  ahead: [],
  routines: [],
  lists: [],
  tags: [],
  counts: {},
  locale: "en",
  agents: {},
};

function answer(cmd: string): Promise<unknown> {
  switch (cmd) {
    case "settle_in":
      return Promise.resolve({ ran: false, brought: false, agrees: true });
    case "docs":
      return Promise.resolve({ folders: [], docs: [] });
    case "sync_state":
      return Promise.resolve({ asked: true });
    case "snapshot":
      return Promise.resolve(shot);
    case "settings":
      return Promise.resolve({ quiet: [], holds: {}, shares: false, onlySharedAbove: 0 });
    case "owed":
      return Promise.resolve([]);
    case "task_of":
      return ipc.task();
    case "help_pages":
      return Promise.resolve([]);
    case "help_reference_here":
    case "help_reference":
      return Promise.resolve("");
    default:
      return Promise.resolve(null);
  }
}

const shown = () => ipc.calls.filter((one) => one.cmd === "brainsty_show");

const opened = async () => {
  const user = userEvent.setup();
  render(<App />);
  await screen.findByText("buy bread");
  return user;
};

const neuron = (kind: Neuron["kind"], id = "x", title = "x"): Neuron => ({ id, kind, title });

const hands = () => ({
  setChosen: vi.fn(),
  choose: vi.fn(),
  opening: vi.fn(),
  setError: vi.fn(),
});

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  ipc.task = () => Promise.resolve(bread);
  heard.on.clear();
});

describe("Brainsty from the main window", () => {
  it("opens the network in its own window, leaving the main window as it was", async () => {
    const user = await opened();

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    expect(shown()).toEqual([{ cmd: "brainsty_show", args: { near: null } }]);
    expect(screen.queryByRole("dialog", { name: t("brainsty") })).toBeNull();
    expect(screen.getByText("buy bread")).toBeTruthy();
  });

  it("opens it near the task open beside the list", async () => {
    const user = await opened();
    await user.click(screen.getByText("buy bread"));
    await screen.findByRole("button", { name: t("expand") });

    await user.click(screen.getByRole("button", { name: t("brainsty") }));

    expect(shown()[0].args).toEqual({ near: "01A" });
    expect(screen.getByRole("button", { name: t("expand") })).toBeTruthy();
  });

  it("opens beside the list a task the network asked for", async () => {
    await opened();

    await waitFor(() => expect(heard.on.has("brainsty-open")).toBe(true));
    act(() => heard.on.get("brainsty-open")?.({ payload: neuron("task", "01A", "buy bread") }));

    expect(await screen.findByRole("button", { name: t("expand") })).toBeTruthy();
  });

  it("stops hearing the network once the window is gone", async () => {
    const { unmount } = render(<App />);
    await screen.findByText("buy bread");
    await waitFor(() => expect(heard.on.has("brainsty-open")).toBe(true));

    unmount();
    await waitFor(() => expect(heard.on.has("brainsty-open")).toBe(false));
  });
});

describe("what the network asks the main window to open", () => {
  it("opens a document or a page among the documents", () => {
    const doc = hands();
    openedHere(neuron("doc", "notes.md"), doc);
    openedHere(neuron("page", "notes-0002.md"), doc);

    expect(doc.setChosen.mock.calls).toEqual([
      [{ named: "docs", doc: "notes.md" }],
      [{ named: "docs", doc: "notes-0002.md" }],
    ]);
  });

  it("opens a tag among the tags by its name", () => {
    const tag = hands();
    openedHere(neuron("tag", "#home", "home"), tag);

    expect(tag.choose).toHaveBeenCalledWith({ named: "tags", tags: ["home"] });
  });

  it("opens a task where it lives, and says why when it cannot be read", async () => {
    const task = hands();
    openedHere(neuron("task", "01A"), task);
    await waitFor(() => expect(task.opening).toHaveBeenCalledWith(bread));
    expect(task.setChosen).toHaveBeenCalledWith({ named: "tasks", slice: "all" });

    ipc.task = () => Promise.reject({ code: "noSuchTask" });
    const gone = hands();
    openedHere(neuron("task", "01Z"), gone);
    await waitFor(() => expect(gone.setError).toHaveBeenCalled());
    expect(gone.opening).not.toHaveBeenCalled();
  });

  it("leaves a file where it is", () => {
    const file = hands();
    openedHere(neuron("file", "attachments/rina.jpg"), file);

    expect(
      [file.setChosen, file.choose, file.opening, file.setError].flatMap((one) => one.mock.calls),
    ).toEqual([]);
  });
});
