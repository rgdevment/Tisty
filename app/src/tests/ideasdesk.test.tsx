import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot } from "../core";
import { t } from "../locales";

Object.defineProperty(window, "innerWidth", { configurable: true, value: 1440 });

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  begun: (): Promise<unknown> => Promise.resolve(SHEET),
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

const SHEET = "a3f1-0007";

const shot: Snapshot = {
  tasks: [],
  total: 0,
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
    case "help_pages":
      return Promise.resolve([]);
    case "brainstorm_begun":
      return ipc.begun();
    case "help_reference_here":
    case "help_reference":
      return Promise.resolve("");
    default:
      return Promise.resolve(null);
  }
}

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const library = async () => {
  const user = userEvent.setup();
  render(<App />);
  const row = await screen.findByRole("button", { name: t("brainstorm") });
  return { user, row };
};

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  ipc.begun = () => Promise.resolve(SHEET);
  heard.on.clear();
});

describe("Ideas in a library that holds nothing yet", () => {
  it("makes the sheet when opened, in the window's language, and looks at the library again", async () => {
    const { user, row } = await library();
    const docsAsked = asked("docs").length;

    await user.click(row);

    await waitFor(() =>
      expect(asked("brainstorm_begun")).toEqual([
        { cmd: "brainstorm_begun", args: { title: t("brainstorm") } },
      ]),
    );
    await waitFor(() => expect(asked("docs").length).toBeGreaterThan(docsAsked));
  });

  it("says why when the sheet cannot be made", async () => {
    ipc.begun = () => Promise.reject({ code: "cannotWrite" });
    const { user, row } = await library();

    await user.click(row);

    expect(await screen.findByRole("alert")).toBeTruthy();
  });

  it("hides from its own menu before it holds anything", async () => {
    const { user, row } = await library();

    await user.pointer({ keys: "[MouseRight]", target: row });
    const menu = await screen.findByRole("menu", { name: t("brainstorm") });
    await user.click(within(menu).getByRole("menuitem", { name: t("hideFromSidebar") }));

    await waitFor(() => expect(screen.queryByRole("button", { name: t("brainstorm") })).toBeNull());
  });
});
