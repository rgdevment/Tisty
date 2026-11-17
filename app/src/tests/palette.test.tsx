import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Found, Snapshot, Task } from "../core";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", async () => ({
  invoke: (await import("./opened")).opening((cmd: string, args?: Record<string, unknown>) =>
    ipc.answer(cmd, args ?? {}),
  ),
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: () => Promise.resolve(() => {}),
  }),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
  }),
}));

const bank: Task = {
  id: "01B",
  title: "call the bank about the watering",
  status: "open",
  priority: "unset",
  order: "a1",
  steps: [],
  log: [],
  volume: {},
};

const shot = (): Snapshot => ({
  tasks: [bank],
  total: 1,
  soonest: {},
  ahead: [],
  routines: [],
  lists: [],
  tags: [],
  counts: {},
  locale: "en",
  agents: {},
});

const paper = (id: string, title: string, wrote: string) => ({
  id,
  file: id,
  title,
  folder: null,
  archived: false,
  away: false,
  wrote,
});

let hits: Found;
let read: string[];

beforeEach(() => {
  localStorage.clear();
  read = [];
  hits = { tasks: [], papers: [], total: 0 };
  ipc.answer = (cmd, args) => {
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({
          folders: [],
          docs: [
            paper("mac0-0001", "Garden plan", "2026-11-01T10:00:00Z"),
            paper("mac0-0002", "Watering", "2026-11-15T10:00:00Z"),
          ],
        });
      case "sync_state":
        return Promise.resolve({ asked: true });
      case "snapshot":
        return Promise.resolve(shot());
      case "search":
        return Promise.resolve(hits);
      case "doc_read":
        read.push(String(args.id ?? ""));
        return Promise.resolve("# Watering\n\nchange the garden hose\n");
      default:
        return Promise.resolve(null);
    }
  };
});

const opened = async () => {
  render(<App />);
  await screen.findByText("call the bank about the watering");
  // The window's keys are heard once its effects have run, a moment after it shows.
  await act(() => Promise.resolve());
};

const press = (shift = false) =>
  fireEvent.keyDown(window, { key: shift ? "P" : "p", ctrlKey: true, shiftKey: shift });

const field = () => screen.getByRole("combobox");

describe("the palette, opened from the keyboard", () => {
  it("opens on Ctrl+P instead of printing, with the documents written last", async () => {
    await opened();

    const printed = !press();

    expect(printed).toBe(true);
    expect(document.activeElement).toBe(field());
    await waitFor(() =>
      expect(screen.getAllByRole("option").map((one) => one.textContent)).toEqual([
        "▤Watering",
        "▤Garden plan",
      ]),
    );
  });

  it("opens on its actions with Ctrl+Shift+P", async () => {
    await opened();

    press(true);

    expect((field() as HTMLInputElement).value).toBe(">");
    expect(screen.getByRole("option", { name: /Priorities/ })).toBeTruthy();
  });

  it("jumps to the document chosen with the keys", async () => {
    const user = userEvent.setup();
    await opened();
    press();

    await user.keyboard("garden{Enter}");

    await waitFor(() => expect(read).toContain("mac0-0001"));
    expect(screen.queryByRole("combobox")).toBeNull();
  });

  it("opens a task the store found", async () => {
    hits = {
      tasks: [{ ...bank, id: "01C", title: "water the ferns", list: undefined }],
      papers: [],
      total: 1,
    };
    const user = userEvent.setup();
    await opened();
    press();

    await user.keyboard("ferns");
    await user.click(await screen.findByRole("option", { name: /water the ferns/ }));

    expect(screen.queryByRole("combobox")).toBeNull();
    expect(await screen.findByDisplayValue("water the ferns")).toBeTruthy();
  });

  it("goes away with Esc and leaves the task open behind it", async () => {
    hits = {
      tasks: [{ ...bank, id: "01C", title: "water the ferns", list: undefined }],
      papers: [],
      total: 1,
    };
    const user = userEvent.setup();
    await opened();
    press();
    await user.keyboard("ferns");
    await user.click(await screen.findByRole("option", { name: /water the ferns/ }));
    await screen.findByDisplayValue("water the ferns");

    press();
    await user.keyboard("{Escape}");

    expect(screen.queryByRole("combobox")).toBeNull();
    expect(screen.getByDisplayValue("water the ferns")).toBeTruthy();
  });

  it("runs an action and goes away, landing where the sidebar's Library lands", async () => {
    const user = userEvent.setup();
    await opened();
    press(true);

    await user.keyboard("library{Enter}");

    expect(screen.queryByRole("combobox")).toBeNull();
    expect(await screen.findByRole("heading", { name: "Unfiled" })).toBeTruthy();
  });

  it("never offers what the store found for words no longer asked", async () => {
    hits = {
      tasks: [{ ...bank, id: "01C", title: "water the ferns", list: undefined }],
      papers: [],
      total: 1,
    };
    const user = userEvent.setup();
    await opened();
    press();
    await user.keyboard("ferns");
    await screen.findByRole("option", { name: /water the ferns/ });
    hits = { tasks: [], papers: [], total: 0 };

    await user.keyboard("x");

    expect(screen.queryByRole("option", { name: /water the ferns/ })).toBeNull();
  });

  it("reaches the finder from Settings when the sidebar was kept folded", async () => {
    const user = userEvent.setup();
    await opened();
    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    await user.click(screen.getByRole("button", { name: "Settings" }));
    press(true);

    await user.keyboard("search tasks{Enter}");

    await waitFor(() => expect(document.activeElement?.id).toBe("side-finder"));
  });

  it("says so when nothing goes by the name", async () => {
    const user = userEvent.setup();
    await opened();
    press();

    await user.keyboard("zzz");

    expect(await screen.findByText(/Nothing goes by that name/)).toBeTruthy();
  });

  it("goes away with Esc and gives the focus back", async () => {
    const user = userEvent.setup();
    await opened();
    const finder = screen.getByRole("textbox", { name: /search tasks/i });
    finder.focus();
    press();

    await user.keyboard("{Escape}");

    expect(screen.queryByRole("combobox")).toBeNull();
    expect(document.activeElement).toBe(finder);
  });

  it("stays shut under another window's question, and still never prints", async () => {
    await opened();
    const veil = document.createElement("div");
    veil.setAttribute("aria-modal", "true");
    document.body.append(veil);

    const printed = !press();
    veil.remove();

    expect(printed).toBe(true);
    expect(screen.queryByRole("combobox")).toBeNull();
  });
});
