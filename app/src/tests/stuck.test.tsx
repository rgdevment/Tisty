import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Settling, Snapshot } from "../core";

const ipc = vi.hoisted(() => ({
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => ipc.answer(cmd, args ?? {}),
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

const shot = (): Snapshot => ({
  tasks: [
    {
      id: "01B",
      title: "call the bank",
      status: "open",
      priority: "unset",
      order: "a1",
      steps: [],
      log: [],
      volume: {},
    },
  ],
  lists: [],
  tags: [],
  refs: [],
  counts: {},
  locale: "en",
});

let settling: Settling;

beforeEach(() => {
  localStorage.clear();
  settling = { ran: true, brought: false, agrees: true };
  ipc.answer = (cmd) => {
    switch (cmd) {
      case "settle_in":
        return Promise.resolve(settling);
      case "docs":
        return Promise.resolve({ folders: [], docs: [] });
      case "sync_state":
        return Promise.resolve({ asked: true, backsUp: true, loose: 0 });
      case "snapshot":
        return Promise.resolve(shot());
      default:
        return Promise.resolve(null);
    }
  };
});

describe("a shared folder that belongs to another history", () => {
  it("says so on opening instead of leaving the machine quietly alone", async () => {
    settling = {
      ran: true,
      brought: false,
      agrees: true,
      stuck: { code: "wouldReset", name: "01M01PS9GC6G5996QPPGBXHR78" },
    };
    render(<App />);

    const said = await screen.findByRole("alert");
    expect(said.textContent).toMatch(/nothing is syncing/i);
  });

  it("does not put a store identifier in front of a person", async () => {
    settling = {
      ran: true,
      brought: false,
      agrees: true,
      stuck: { code: "wouldReset", name: "01M01PS9GC6G5996QPPGBXHR78" },
    };
    render(<App />);

    const said = await screen.findByRole("alert");
    expect(said.textContent).not.toContain("01M01PS9GC");
  });

  it("says what can be done about it, not only that it happened", async () => {
    settling = {
      ran: true,
      brought: false,
      agrees: true,
      stuck: { code: "wouldReset", name: "01M01PS9GC6G5996QPPGBXHR78" },
    };
    render(<App />);

    const said = await screen.findByRole("alert");
    expect(said.textContent).toMatch(/settings/i);
    expect(said.textContent).toMatch(/backed up/i);
  });

  it("says the same thing whichever of the two refusals came back", async () => {
    settling = {
      ran: true,
      brought: false,
      agrees: true,
      stuck: { code: "otherStore", name: "01M01PS9GC6G5996QPPGBXHR78" },
    };
    render(<App />);

    const said = await screen.findByRole("alert");
    expect(said.textContent).toMatch(/nothing is syncing/i);
    expect(said.textContent).not.toContain("01M01PS9GC");
  });

  it("still speaks when the folder refuses for some other reason", async () => {
    settling = {
      ran: true,
      brought: false,
      agrees: true,
      stuck: { code: "notAllowed", name: "dev_ej8mf31b" },
    };
    render(<App />);

    expect((await screen.findByRole("alert")).textContent).toBeTruthy();
  });

  it("stays quiet when the carry went through", async () => {
    render(<App />);
    await screen.findByText("call the bank");

    expect(screen.queryByRole("alert")).toBeNull();
  });
});

describe("a store written by a newer Tisty", () => {
  it("says so plainly instead of handing over a technical line", async () => {
    settling = { ran: true, brought: false, agrees: true, stuck: { code: "storeNewer" } };
    render(<App />);

    const said = await screen.findByRole("alert");
    expect(said.textContent).toMatch(/newer Tisty/i);
    expect(said.textContent).not.toMatch(/schema/i);
  });

  it("offers the update in the same breath", async () => {
    const asked: string[] = [];
    const was = ipc.answer;
    ipc.answer = (cmd, args) => {
      asked.push(cmd);
      if (cmd === "update_ready") {
        return Promise.resolve({
          version: "9.9.9",
          route: "download",
          package: null,
          installs: true,
        });
      }
      return was(cmd, args);
    };
    settling = { ran: true, brought: false, agrees: true, stuck: { code: "storeNewer" } };
    render(<App />);

    const said = await screen.findByRole("alert");
    await userEvent.click(within(said).getByRole("button", { name: /^update$/i }));

    expect(asked).toContain("update_install");
  });
});
