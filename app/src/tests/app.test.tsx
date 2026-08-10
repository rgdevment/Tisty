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
  invoke: (cmd: string, args: Record<string, unknown>) => ipc.answer(cmd, args),
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

const shot = (view: { archive?: boolean } | undefined): Snapshot => ({
  tasks: tasks.filter((one) => (view?.archive ? one.status !== "open" : one.status === "open")),
  lists: [],
  tags: [],
  refs: [],
  counts: {},
  locale: "en",
});

beforeEach(() => {
  localStorage.clear();
  tasks = structuredClone([report, bank, filed]);
  ipc.answer = (cmd, args) => {
    const held = tasks.find((one) => one.id === args.id);
    switch (cmd) {
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

    await user.click(screen.getByRole("button", { name: "write the report" }));

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

  it("stays put in the side panel, where the list never left", async () => {
    const user = userEvent.setup();
    await started();

    await user.click(screen.getByText("write the report"));
    await user.click(screen.getByRole("button", { name: /Not doing it/ }));

    await screen.findByRole("button", { name: /Reopen/ });
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

    await user.click(screen.getByRole("button", { name: "write the report" }));

    expect(await screen.findByText("That is not a task")).toBeTruthy();
  });
});
