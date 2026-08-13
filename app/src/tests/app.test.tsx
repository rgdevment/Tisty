import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Snapshot, Task } from "../core";
import App from "../App";

const ipc = vi.hoisted(() => ({
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> =>
    Promise.resolve(null),
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

const report: Task = {
  id: "01A",
  title: "write the report",
  status: "open",
  priority: 4,
  order: "a0",
  steps: [{ id: "01S", text: "collect the figures", done: false, order: "a0" }],
  log: [],
  volume: { steps: 1, steps_done: 0 },
};

const bank: Task = {
  id: "01B",
  title: "call the bank",
  status: "open",
  priority: 4,
  order: "a1",
  steps: [],
  log: [],
  volume: {},
};

const filed: Task = {
  id: "01C",
  title: "filed last month",
  status: "done",
  priority: 4,
  order: "a2",
  completed_at: "2026-07-04 10:00:00",
  volume: {},
};

let tasks: Task[];
let counts: Record<string, number>;

const shot = (view: { archive?: boolean } | undefined): Snapshot => ({
  tasks: tasks.filter((one) => (view?.archive ? one.status !== "open" : one.status === "open")),
  lists: [],
  tags: [],
  refs: [],
  counts,
  locale: "en",
});

beforeEach(() => {
  localStorage.clear();
  tasks = structuredClone([report, bank, filed]);
  counts = {};
  ipc.answer = (cmd, args) => {
    const held = tasks.find((one) => one.id === args.id);
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: false, brought: false, agrees: true });
      case "docs":
        return Promise.resolve([]);
      case "sync_state":
        return Promise.resolve({ asked: true, backsUp: true, loose: 0 });
      case "snapshot":
        return Promise.resolve(shot(args.view as { archive?: boolean } | undefined));
      case "read":
        return Promise.resolve({ title: String(args.text ?? ""), tags: [], spans: [], offers: [] });
      case "complete":
        held!.status = "done";
        return Promise.resolve(held);
      case "discard":
        held!.status = "dropped";
        held!.hidden = true;
        return Promise.resolve(held);
      default:
        return Promise.resolve(held);
    }
  };
});

const started = async () => {
  render(<App />);
  await screen.findByText("write the report");
};

const value = (name: string) =>
  (screen.getByRole("textbox", { name }) as HTMLTextAreaElement | HTMLInputElement).value;

describe("the open panel", () => {
  it("stays on a task the action pushed out of the view", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    await user.click(screen.getByRole("button", { name: "Complete write the report" }));

    await screen.findByRole("button", { name: /Reopen/ });
    expect(value("Title")).toBe("write the report");
  });

  it("does not carry a half-written step from one task to the next", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await user.type(await screen.findByRole("textbox", { name: "Add a step" }), "half a thought");
    expect(value("Add a step")).toBe("half a thought");

    await user.click(screen.getByText("call the bank"));
    await waitFor(() => expect(value("Title")).toBe("call the bank"));
    expect(value("Add a step")).toBe("");
  });
});

describe("the full screen", () => {
  it("lets go of a task once you say you will not do it", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await user.click(screen.getByRole("button", { name: /Full screen/ }));
    await screen.findByRole("button", { name: /Not doing it/ });

    await user.click(screen.getByRole("button", { name: /Not doing it/ }));

    await waitFor(() => expect(screen.queryByRole("textbox", { name: "Title" })).toBeNull());
    expect(screen.getByText("call the bank")).toBeTruthy();
  });

  /// Asked for by the author: the task leaves the list, so the column showing
  /// it has nothing left to show. The cost is real and accepted — the Reopen
  /// button went away with it, so undoing a mistaken discard now means finding
  /// the task in the archive.
  it("closes the side panel too, once the task has left the list", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await user.click(screen.getByRole("button", { name: /Not doing it/ }));

    await waitFor(() => expect(screen.queryByRole("textbox", { name: "Title" })).toBeNull());
  });
});

describe("the archive", () => {
  it("draws no completing circle, where a click would rewrite history", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByRole("button", { name: /Archive/ }));
    await screen.findByText("filed last month");

    expect(screen.queryByRole("button", { name: "filed last month" })).toBeNull();
    expect(screen.queryByText("write the report")).toBeNull();
  });
});

describe("a refusal", () => {
  it("reaches the window instead of failing silently", async () => {
    const user = userEvent.setup();
    await started();

    ipc.answer = (cmd, args) =>
      cmd === "snapshot"
        ? Promise.resolve(shot(args.view as { archive?: boolean } | undefined))
        : Promise.reject({ code: "notATaskId" });

    await user.click(screen.getByRole("button", { name: "Complete write the report" }));

    expect(await screen.findByText("That is not a task")).toBeTruthy();
  });
});

describe("what the numbers on screen refer to", () => {
  /// One number beside a title that always says «Tasks» reads as the whole of
  /// them, while it only ever counted the slice on show — and the chips said
  /// nothing about what was behind them.
  it("puts each count on its own chip, and none beside the title", async () => {
    counts = { tasks: 2, upcoming: 5, repeating: 1, all: 8 };
    await started();

    expect(screen.getByRole("button", { name: /today/i }).textContent).toContain("2");
    expect(screen.getByRole("button", { name: /upcoming/i }).textContent).toContain("5");
    expect(screen.getByRole("button", { name: /^all/i }).textContent).toContain("8");
    expect(screen.getByRole("heading", { level: 1 }).parentElement?.textContent).toBe("Tasks");
  });

  /// The archive is one view with one number, so there it still belongs there.
  it("keeps the count beside a title that names one list", async () => {
    await started();

    await userEvent.click(screen.getByRole("button", { name: /Archive/ }));
    await screen.findByText("filed last month");

    expect(screen.getByRole("heading", { level: 1 }).parentElement?.textContent).toBe("Archive1");
  });
});

describe("closing an open task", () => {
  /// It dropped the keyboard on the body, so the next Tab started over from the
  /// top of the window.
  it("hands the keyboard back to the row it was opened from", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    await user.click(screen.getByRole("button", { name: /close the task/i }));

    await waitFor(() => expect(screen.queryByRole("textbox", { name: "Title" })).toBeNull());
    expect(document.activeElement).toBe(document.querySelector('[data-row="01A"]'));
  });
});

describe("opening a task beside the list", () => {
  /// The list lost its centring and its column narrowed in the same frame, so
  /// it jumped left and the row that had just been clicked slid out from under
  /// the cursor.
  it("does not throw the list to the left", async () => {
    const user = userEvent.setup();
    await started();
    const list = screen.getByRole("list");
    expect(list.className).toContain("mx-auto");

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    expect(list.className).toContain("mx-auto");
  });
});
