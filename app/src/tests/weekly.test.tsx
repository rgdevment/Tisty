import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import type { Filed, List, Snapshot, Task } from "../core";
import type { Reviewing } from "../reviewing";

const DAY = 86_400;

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args?: Record<string, unknown> }[],
  review: null as unknown,
  docs: [] as unknown[],
  forgotten: [] as unknown[],
  ideas: 0,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args });
    switch (cmd) {
      case "settle_in":
        return Promise.resolve({ ran: true, brought: false, agrees: true });
      case "docs":
        return Promise.resolve({ folders: [], docs: ipc.docs });
      case "sync_state":
        return Promise.resolve({ asked: true });
      case "snapshot":
        return Promise.resolve(shot());
      case "review_of":
        return Promise.resolve(ipc.review);
      case "docs_forgotten":
        return Promise.resolve(ipc.forgotten);
      case "brainstorm_of":
        return Promise.resolve(
          ipc.ideas ? { file: "hevp3wf9-0009", ideas: ipc.ideas, marked: 0 } : null,
        );
      case "review_remind":
      case "complete":
        return Promise.resolve(task("01R1", "Weekly review"));
      default:
        return Promise.resolve(null);
    }
  },
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

const task = (id: string, title: string, more: Partial<Task> = {}): Task => ({
  id,
  title,
  status: "open",
  priority: "unset",
  order: id,
  steps: [],
  log: [],
  volume: {},
  ...more,
});

const lists: List[] = [
  { id: "L1", name: "Personal", order: "a" },
  { id: "L2", name: "Casa", order: "b" },
];

const shot = (): Snapshot => ({
  tasks: [],
  total: 0,
  soonest: {},
  ahead: [],
  routines: [],
  lists,
  tags: [],
  refs: [],
  counts: {},
  locale: "en",
  agents: {},
});

const paper = (id: string, title: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `hevp3wf9-${id}`,
  title,
  folder: null,
  archived: false,
  away: false,
  ...more,
});

const reviewing = (more: Partial<Reviewing> = {}): Reviewing => ({
  routine: null,
  unlisted: [],
  ...more,
});

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const openReview = async () => {
  render(<App />);
  await userEvent.click(await screen.findByRole("button", { name: "Weekly review" }));
  return screen.findByRole("heading", { level: 1, name: "Weekly review" });
};

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  ipc.review = reviewing();
  ipc.docs = [];
  ipc.forgotten = [];
  ipc.ideas = 0;
});

describe("the weekly review", () => {
  it("walks what waits to be decided, what was marked as past and what was forgotten", async () => {
    ipc.review = reviewing({ unlisted: [task("01T1", "Call Jaime")] });
    ipc.ideas = 2;
    ipc.docs = [
      paper("0001", "Old receipts", {
        flagged: { at: "2026-06-01T00:00:00Z", said: "A copy of Receipts" },
      }),
      paper("0002", "Grandma's recipes"),
    ];
    ipc.forgotten = [{ doc: "0002", last: Date.now() / 1000 - 200 * DAY, lastOpenedHere: false }];

    await openReview();

    const decide = screen.getByRole("region", { name: "To decide" });
    expect(within(decide).getByRole("button", { name: "Call Jaime" })).toBeTruthy();
    expect(within(decide).getByRole("button", { name: "Open Brainstorm, 2 waiting" })).toBeTruthy();
    const marked = screen.getByRole("region", { name: "Marked as past" });
    expect(within(marked).getByText("A copy of Receipts")).toBeTruthy();
    const forgotten = await screen.findByRole("region", { name: "Not opened in a while" });
    expect(
      await within(forgotten).findByRole("button", { name: "Grandma's recipes" }),
    ).toBeTruthy();
  });

  it("marks a stop with nothing in it as done", async () => {
    await openReview();

    const marked = screen.getByRole("region", { name: "Marked as past" });
    expect(within(marked).getByRole("img", { name: "Nothing waits" })).toBeTruthy();
    expect(within(marked).getByText("Nothing waits here.")).toBeTruthy();
  });

  it("archives a marked document, and completes or drops a task from where it stands", async () => {
    ipc.review = reviewing({ unlisted: [task("01T1", "Call Jaime")] });
    ipc.docs = [
      paper("0001", "Old receipts", { flagged: { at: "2026-06-01T00:00:00Z", said: "Old" } }),
    ];
    await openReview();

    const marked = screen.getByRole("region", { name: "Marked as past" });
    await userEvent.click(within(marked).getByRole("button", { name: "Archive" }));
    const decide = screen.getByRole("region", { name: "To decide" });
    await userEvent.click(within(decide).getByRole("button", { name: "Complete" }));

    await waitFor(() => expect(asked("doc_away")[0]?.args).toEqual({ id: "0001", away: true }));
    await waitFor(() => expect(asked("complete")[0]?.args).toMatchObject({ id: "01T1" }));
  });

  it.each([
    ["as a sheet", null],
    ["beside the page", "columns"],
  ])("opens an unlisted task's detail %s", async (_, detail) => {
    if (detail) localStorage.setItem("detail", detail);
    ipc.review = reviewing({ unlisted: [task("01T1", "Call Jaime")] });
    await openReview();

    const decide = screen.getByRole("region", { name: "To decide" });
    await userEvent.click(within(decide).getByRole("button", { name: "Open" }));

    const titled = await screen.findByRole("textbox", { name: "Title" });
    expect((titled as HTMLInputElement).value ?? titled.textContent).toContain("Call Jaime");
  });

  it("offers a weekly reminder on Friday at five in the first list, and files it as asked", async () => {
    await openReview();

    const nudge = screen.getByRole("region", { name: "Remind me every week" });
    expect((within(nudge).getByRole("combobox", { name: "Day" }) as HTMLSelectElement).value).toBe(
      "5",
    );
    await userEvent.selectOptions(within(nudge).getByRole("combobox", { name: "List" }), "L2");
    await userEvent.click(within(nudge).getByRole("button", { name: "Remind me every week" }));

    await waitFor(() =>
      expect(asked("review_remind")[0]?.args).toEqual({
        nudge: {
          title: "Weekly review",
          tag: "weekly-review",
          list: "L2",
          weekday: 5,
          hour: 17,
          minute: 0,
        },
      }),
    );
  });

  it("with a routine set, finishing the review closes this round", async () => {
    ipc.review = reviewing({
      routine: task("01R1", "Weekly review", {
        date: { at: "2026-06-12T00:00:00", tz: "UTC", floating: true, has_time: false },
        repeat: { from: "due", each: { every: 1, unit: "week" } },
      }),
    });
    await openReview();

    expect(screen.queryByRole("region", { name: "Remind me every week" })).toBeNull();
    expect(screen.getByText(/^Due /)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Finish the review" }));

    await waitFor(() => expect(asked("complete")[0]?.args).toMatchObject({ id: "01R1" }));
  });
});
