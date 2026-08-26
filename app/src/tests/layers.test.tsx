import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot, Task, View } from "../core";

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

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  save: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
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

const open: Task = {
  id: "01A",
  title: "write the report",
  status: "open",
  priority: "unset",
  order: "a0",
  volume: {},
};

const told: Task = {
  id: "01B",
  title: "renew the certificate",
  status: "done",
  priority: "unset",
  order: "a1",
  completed_at: "2026-08-12T10:00:00Z",
  volume: { journal: 1, prose: 1 } as Task["volume"],
};

const errand: Task = {
  id: "01C",
  title: "buy bread",
  status: "done",
  priority: "unset",
  order: "a2",
  completed_at: "2026-08-03T10:00:00Z",
  volume: {},
};

const shot = (view: View | undefined): Snapshot => ({
  tasks: view?.archive ? (view.reading === "trace" ? [errand] : [told]) : [open],
  lists: [],
  tags: [],
  refs: [],
  counts: { stories: 1, routines: 0, traces: 1 },
  locale: "en",
});

const views = () =>
  ipc.calls.filter((one) => one.cmd === "snapshot").map((one) => one.args.view as View | undefined);

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  ipc.answer = (cmd, args) => {
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: [] });
      case "sync_state":
        return Promise.resolve({ asked: true, backsUp: true, loose: 0 });
      case "snapshot":
        return Promise.resolve(shot(args.view as View | undefined));
      case "routines":
        return Promise.resolve([]);
      case "archive_shape":
        return Promise.resolve({
          closed: 2,
          dropped: 0,
          told: 1,
          since: "2026-07-01T09:00:00Z",
          months: [
            { key: "2026-07", closed: 1 },
            { key: "2026-08", closed: 1 },
          ],
        });
      default:
        return Promise.resolve(null);
    }
  };
});

const inTheArchive = async (user: ReturnType<typeof userEvent.setup>) => {
  render(<App />);
  await screen.findByText("write the report");
  await user.click(screen.getByRole("button", { name: /Archive/ }));
  await screen.findByText("renew the certificate");
};

describe("the archive reads in layers", () => {
  it("opens on what has something to tell, not on everything closed", async () => {
    const user = userEvent.setup();
    await inTheArchive(user);

    expect(views().some((view) => view?.archive && view.reading === "story")).toBe(true);
    expect(views().some((view) => view?.archive && view.reading === undefined)).toBe(false);
  });

  it("keeps the lighter layers reachable, each with its count", async () => {
    const user = userEvent.setup();
    await inTheArchive(user);

    expect(screen.getByRole("button", { name: /Stories/ }).getAttribute("aria-pressed")).toBe(
      "true",
    );
    expect(screen.getByRole("button", { name: /Routines/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Trace/ })).toBeTruthy();
  });

  it("opens with what the archive holds, not with a bare list", async () => {
    const user = userEvent.setup();
    await inTheArchive(user);

    await waitFor(() =>
      expect(screen.getByRole("region", { name: /What the archive holds/i })).toBeTruthy(),
    );
    expect(screen.getByText(/left something written/i)).toBeTruthy();
  });

  it("shows routines as series, never as one row per turn", async () => {
    const user = userEvent.setup();
    await inTheArchive(user);

    await user.click(screen.getByRole("button", { name: /Routines/ }));

    await waitFor(() => expect(screen.getByText(/Nothing repeats yet/i)).toBeTruthy());
    expect(ipc.calls.some((one) => one.cmd === "routines")).toBe(true);
  });

  it("asks for the trace only once it is chosen", async () => {
    const user = userEvent.setup();
    await inTheArchive(user);

    expect(views().some((view) => view?.reading === "trace")).toBe(false);

    await user.click(screen.getByRole("button", { name: /Trace/ }));

    await waitFor(() => expect(screen.getByText("buy bread")).toBeTruthy());
    expect(views().some((view) => view?.reading === "trace")).toBe(true);
  });
});
