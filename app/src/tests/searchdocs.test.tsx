import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Found, Snapshot, Task } from "../core";

const ipc = vi.hoisted(() => ({
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => ipc.answer(cmd, args ?? {}),
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
  priority: 4,
  order: "a1",
  steps: [],
  log: [],
  volume: {},
};

const shot = (): Snapshot => ({
  tasks: [bank],
  lists: [],
  tags: [],
  refs: [],
  counts: {},
  locale: "en",
});

const paper = {
  id: "mac0-0002",
  file: "mac0-0002",
  title: "Watering",
  folder: null,
  archived: false,
};

let hits: Found;
let read: string[];

beforeEach(() => {
  localStorage.clear();
  read = [];
  hits = {
    tasks: [],
    papers: [
      { id: "mac0-0002", title: "Watering", line: "change the garden hose", archived: false },
    ],
    total: 0,
  };
  ipc.answer = (cmd, args) => {
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: [paper] });
      case "sync_state":
        return Promise.resolve({ asked: true, backsUp: true, loose: 0 });
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

const searching = async (user: ReturnType<typeof userEvent.setup>, words: string) => {
  render(<App />);
  await screen.findByText("call the bank about the watering");
  await user.click(screen.getByRole("button", { name: /search/i }));
  await user.type(await screen.findByRole("textbox", { name: /search everywhere/i }), words);
};

describe("a search that reaches the documents", () => {
  it("never shows a file name where a title should be", async () => {
    hits = {
      tasks: [],
      papers: [{ id: "914kqe8z-0004", title: "", line: "change the garden hose", archived: false }],
      total: 0,
    };
    const user = userEvent.setup();
    await searching(user, "hose");

    const line = await screen.findByText("change the garden hose");
    const said = line.closest("button")?.textContent ?? "";
    expect(said).not.toContain("914kqe8z");
    expect(said).toContain("Untitled");
  });

  it("marks a document that was put away, so the archive never passes for the tree", async () => {
    hits = {
      tasks: [],
      papers: [
        { id: "mac0-0002", title: "Watering", line: "change the garden hose", archived: true },
      ],
      total: 0,
    };
    const user = userEvent.setup();
    await searching(user, "watering");

    const line = await screen.findByText("change the garden hose");
    expect(line.closest("button")?.textContent).toMatch(/Watering.*Archived/s);
  });

  it("shows the document beside the tasks, with the line it was found on", async () => {
    const user = userEvent.setup();
    await searching(user, "hose");

    const line = await screen.findByText("change the garden hose");
    expect(line.closest("button")?.textContent).toContain("Watering");
  });

  it("opens the document when the hit is clicked", async () => {
    const user = userEvent.setup();
    await searching(user, "hose");

    const line = await screen.findByText("change the garden hose");
    await user.click(line.closest("button")!);

    await waitFor(() => expect(read).toContain("mac0-0002"));
  });

  it("says the hits are in documents instead of claiming nothing matched", async () => {
    const user = userEvent.setup();
    await searching(user, "hose");

    await screen.findByText(/hits are in documents/i);
    expect(screen.queryByText(/Nothing matched/i)).toBeNull();
  });

  it("keeps the plain empty message when no document matched either", async () => {
    hits = { tasks: [], papers: [], total: 0 };
    const user = userEvent.setup();
    await searching(user, "hose");

    await screen.findByText(/Nothing matched/i);
    expect(screen.queryByText(/hits are in documents/i)).toBeNull();
  });
});
