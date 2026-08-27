import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Snapshot, Task } from "../core";

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

const report: Task = {
  id: "01A",
  title: "write the report",
  status: "open",
  priority: "unset",
  order: "a0",
  steps: [{ id: "01S", text: "collect the figures", done: false, order: "a0" }],
  log: [],
  volume: { steps: 1, steps_done: 0 },
};

const bank: Task = {
  id: "01B",
  title: "call the bank",
  status: "open",
  priority: "unset",
  order: "a1",
  steps: [],
  log: [],
  volume: {},
};

const filed: Task = {
  id: "01C",
  title: "filed last month",
  status: "done",
  priority: "unset",
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
  ipc.calls = [];
  tasks = structuredClone([report, bank, filed]);
  counts = {};
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
      case "task_story":
        return Promise.resolve({ id: String(args.id ?? ""), pages: [] });
      case "read":
        return Promise.resolve({ title: String(args.text ?? ""), tags: [], spans: [], offers: [] });
      case "complete":
        if (held) held.status = "done";
        return Promise.resolve(held);
      case "discard":
        if (held) {
          held.status = "dropped";
          held.hidden = true;
        }
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
  it("lets go of the task once it is completed from the list", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    await user.click(screen.getByRole("button", { name: "Complete write the report" }));

    await waitFor(() => expect(screen.queryByRole("textbox", { name: "Title" })).toBeNull());
  });

  it("stays on a task it did not complete", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    await user.click(screen.getByRole("button", { name: "Complete call the bank" }));

    expect(value("Title")).toBe("write the report");
  });

  it("lets go of the task once it is completed from the panel itself", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    await user.click(await screen.findByRole("button", { name: /^Complete$/ }));

    await waitFor(() => expect(screen.queryByRole("textbox", { name: "Title" })).toBeNull());
  });

  it("does not leave the panel showing a task settled after it was reopened", async () => {
    const user = userEvent.setup();
    const quick = ipc.answer;
    ipc.answer = (cmd, args) => {
      if (cmd === "snapshot") {
        return new Promise((go) =>
          setTimeout(() => go(shot(args.view as { archive?: boolean })), 30),
        );
      }
      if (cmd === "reopen") {
        const at = tasks.findIndex((one) => one.id === args.id);
        if (at < 0) return Promise.resolve(null);
        const back: Task = { ...tasks[at], status: "open" };
        tasks[at] = back;
        return Promise.resolve(back);
      }
      return quick(cmd, args);
    };
    await started();

    await user.click(screen.getByRole("button", { name: /Archive/ }));
    await user.click(await screen.findByText("filed last month"));

    await user.click(await screen.findByRole("button", { name: /Reopen/ }));

    expect(await screen.findByRole("button", { name: /^Complete$/ })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /Reopen/ })).toBeNull();
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

  it("closes the side panel too, once the task has left the list", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await user.click(await screen.findByRole("button", { name: /Not doing it/ }));

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
  it("puts each count on its own chip, and none beside the title", async () => {
    counts = { tasks: 2, upcoming: 5, repeating: 1, all: 8 };
    await started();

    expect(screen.getByRole("button", { name: /today/i }).textContent).toContain("2");
    expect(screen.getByRole("button", { name: /upcoming/i }).textContent).toContain("5");
    expect(screen.getByRole("button", { name: /^all/i }).textContent).toContain("8");
    expect(screen.getByRole("heading", { level: 1 }).parentElement?.textContent).toBe("Tasks");
  });

  it("keeps the count beside a title that names one list", async () => {
    await started();

    await userEvent.click(screen.getByRole("button", { name: /Archive/ }));
    await screen.findByText("filed last month");

    expect(screen.getByRole("heading", { level: 1 }).parentElement?.textContent).toBe("Archive1");
  });
});

describe("closing an open task", () => {
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
  it("does not throw the list to the left", async () => {
    const user = userEvent.setup();
    await started();
    const list = screen.getByRole("list", { name: "Tasks" });
    expect(list.className).toContain("mx-auto");

    await user.click(screen.getByText("write the report"));
    await screen.findByRole("textbox", { name: "Title" });

    expect(list.className).toContain("mx-auto");
  });
});

describe("what a sync brings in", () => {
  it("puts a document that arrived from another machine on the list", async () => {
    const plain = ipc.answer;
    ipc.answer = (cmd, args) => {
      switch (cmd) {
        case "sync_state":
          return Promise.resolve({
            chosen: "G:/My Drive/tisty",
            asked: true,
            backsUp: false,
            loose: 0,
          });
        case "sync_now":
          return Promise.resolve({ carried: "came", undecided: [], unreadable: [] });
        default:
          return plain(cmd, args);
      }
    };
    await started();

    await waitFor(() =>
      expect(ipc.calls.filter((one) => one.cmd === "sync_now").length).toBeGreaterThan(0),
    );

    await waitFor(() =>
      expect(ipc.calls.filter((one) => one.cmd === "docs").length).toBeGreaterThan(1),
    );
  });
});

describe("what the maintenance screen writes", () => {
  const withMachines = () => {
    const plain = ipc.answer;
    ipc.answer = (cmd, args) => {
      switch (cmd) {
        case "sync_state":
          return Promise.resolve({
            chosen: "G:/My Drive/tisty",
            asked: true,
            backsUp: false,
            loose: 0,
          });
        case "sync_now":
          return Promise.resolve({ carried: "same", undecided: [], unreadable: [] });
        case "reachable":
          return Promise.resolve({ shipped: true, withinReach: false, onPath: true });
        case "checked":
          return Promise.resolve({
            tasks: 1,
            lists: 0,
            agrees: true,
            loose: 0,
            looseBytes: 0,
            astray: [],
            events: 1,
            machines: [
              { id: "mac0-0001", when: Math.floor(Date.now() / 1000), mine: true },
              { id: "win1-0002", when: Math.floor(Date.now() / 1000), mine: false },
            ],
            logBytes: 0,
            docsBytes: 0,
            heldBytes: 0,
            heldFiles: 0,
          });
        case "remove_machine":
          return Promise.resolve(null);
        default:
          return plain(cmd, args);
      }
    };
  };

  const pushes = () => ipc.calls.filter((one) => one.cmd === "sync_now" && one.args.way === "push");

  it("carries a machine removal out without waiting for the quarter-hour beat", async () => {
    const user = userEvent.setup();
    withMachines();
    await started();

    await user.click(screen.getByRole("button", { name: /settings/i }));
    await user.click(await screen.findByRole("tab", { name: /maintenance/i }));
    await user.click(screen.getByRole("button", { name: /^review$/i }));
    await screen.findByText("win1-0002");
    await user.click(screen.getByRole("button", { name: /^remove$/i }));
    await waitFor(() =>
      expect(ipc.calls.filter((one) => one.cmd === "remove_machine")).toHaveLength(1),
    );

    await waitFor(() => expect(pushes()).toHaveLength(1), { timeout: 8_000 });
  }, 15_000);
});
