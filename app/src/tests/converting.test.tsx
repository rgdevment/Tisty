import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import {
  erasable,
  type Page,
  readingOf,
  type Snapshot,
  type Task,
  type View,
  weightOf,
} from "../core";
import Trail from "../ui/Trail";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string, _args: Record<string, unknown>): Promise<unknown> => Promise.resolve(null),
}));

const dialog = vi.hoisted(() => ({ yes: true }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd, args ?? {});
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: () => Promise.resolve(null),
  save: () => Promise.resolve(null),
  ask: () => Promise.resolve(dialog.yes),
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
  volume: { journal: 3, prose: 3 },
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

const counts = { stories: 1, routines: 0, traces: 2, folded: 0, tracesHidden: 0 };

const shot = (view: View | undefined): Snapshot => ({
  tasks: view?.archive
    ? view.hidden
      ? []
      : view.reading === "trace"
        ? counts.traces
          ? [errand]
          : []
        : [told]
    : [open],
  ahead: [],
  routines: [],
  lists: [],
  tags: [],
  refs: [],
  counts,
  locale: "en",
  agents: {},
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(() => {
  localStorage.clear();
  ipc.calls = [];
  dialog.yes = true;
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
          closed: 3,
          dropped: 0,
          told: 1,
          since: "2026-07-01T09:00:00Z",
          months: [{ key: "2026-08", closed: 3 }],
        });
      default:
        return Promise.resolve(null);
    }
  };
});

const inTheTrace = async (user: ReturnType<typeof userEvent.setup>) => {
  render(<App />);
  await screen.findByText("write the report");
  await user.click(screen.getByRole("button", { name: /Archive/ }));
  await screen.findByText("renew the certificate");
  await user.click(screen.getByRole("button", { name: /Trace/ }));
  await screen.findByText("buy bread");
};

describe("the layer a task reads as, decided in the window the way the core decides it", () => {
  // The same buckets the core uses: steps 0–2 → 0, 3–7 → 1, 8+ → 2; refs 0 → 0, 1–2 → 1, 3+ → 2.
  it("weighs prose, then steps and references by bucket", () => {
    expect(weightOf({ ...errand, volume: {} })).toBe(0);
    expect(weightOf({ ...errand })).toBe(0);
    expect(weightOf({ ...errand, volume: { prose: 2, steps: 3, refs: 1 } })).toBe(4);
    expect(weightOf({ ...errand, volume: { prose: 8, steps: 8, refs: 3 } })).toBe(12);
    expect(weightOf({ ...errand, volume: { steps: 2 } })).toBe(0);
    expect(weightOf({ ...errand, volume: { steps: 7 } })).toBe(1);
    expect(weightOf({ ...errand, volume: { steps: 8 } })).toBe(2);
    expect(weightOf({ ...errand, volume: { refs: 2 } })).toBe(1);
    expect(weightOf({ ...errand, volume: { refs: 3 } })).toBe(2);
    expect(readingOf({ ...errand, volume: { prose: 2 } })).toBe("trace");
    expect(readingOf({ ...errand, volume: { prose: 3 } })).toBe("story");
  });

  it("reads a routine as a routine, a conversion as what it says, and the rest by weight", () => {
    expect(readingOf(errand)).toBe("trace");
    expect(readingOf(told)).toBe("story");
    expect(readingOf({ ...errand, read_as: "story" })).toBe("story");
    expect(readingOf({ ...told, read_as: "trace" })).toBe("trace");
    expect(readingOf({ ...told, read_as: "trace", after: "01B" })).toBe("routine");
    expect(
      readingOf({ ...errand, repeat: { from: "due", each: { every: 1, unit: "week" } } }),
    ).toBe("routine");
  });

  it("lets only a closed trace go", () => {
    expect(erasable(errand)).toBe(true);
    expect(erasable({ ...errand, hidden: true })).toBe(true);
    expect(erasable({ ...errand, status: "dropped" })).toBe(true);
    expect(erasable(told)).toBe(false);
    expect(erasable({ ...told, read_as: "trace" })).toBe(true);
    expect(erasable({ ...errand, read_as: "story" })).toBe(false);
    expect(erasable({ ...errand, after: "01B" })).toBe(false);
    expect(erasable(open)).toBe(false);
  });
});

describe("the trace layer acts on one task at a time", () => {
  it("offers no bulk action in any layer", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("write the report");
    await user.click(screen.getByRole("button", { name: /Archive/ }));
    await screen.findByText("renew the certificate");
    await user.click(screen.getByRole("button", { name: /Trace/ }));
    await screen.findByText("buy bread");

    expect(screen.queryByRole("button", { name: /hide all/i })).toBeNull();
    expect(screen.queryByRole("button", { name: /erase all/i })).toBeNull();
  });

  it("says where the trace went once it is all hidden", async () => {
    const user = userEvent.setup();
    counts.traces = 0;
    counts.folded = 41;
    counts.tracesHidden = 40;
    try {
      render(<App />);
      await screen.findByText("write the report");
      await user.click(screen.getByRole("button", { name: /Archive/ }));
      await screen.findByText("renew the certificate");
      await user.click(screen.getByRole("button", { name: /Trace/ }));
      await screen.findByText(/40 under «hidden»/i);
    } finally {
      counts.traces = 2;
      counts.folded = 0;
      counts.tracesHidden = 0;
    }
  });

  it("gives each dense row its own hide button", async () => {
    const user = userEvent.setup();
    await inTheTrace(user);

    await user.click(screen.getByRole("button", { name: /^hide it$/i }));

    await waitFor(() => expect(sent("fold")).toHaveLength(1));
    expect(sent("fold")[0].args).toEqual({ id: "01C", away: true });
  });
});

describe("the trail tells a conversion", () => {
  it("names the layer the person chose, and when it was taken back", async () => {
    const page = (n: number, from: string | null, to: string | null): Page =>
      ({
        n,
        at: "2026-08-12T10:00:00Z",
        by: "dev_a",
        chapter: "converted",
        from,
        to,
      }) as unknown as Page;
    ipc.answer = (cmd) =>
      cmd === "task_story"
        ? Promise.resolve({
            id: "01B",
            pages: [page(1, null, "story"), page(2, "story", "trace"), page(3, "trace", null)],
          })
        : Promise.resolve(null);

    render(<Trail task="01B" lists={[]} />);
    await screen.findByRole("list");

    expect(screen.getByText(/kept as a story/i)).toBeTruthy();
    expect(screen.getByText(/read as a trace/i)).toBeTruthy();
    expect(screen.getByText(/read again by what it holds/i)).toBeTruthy();
  });

  it("tells when the task was let to an agent, and when it was kept again", async () => {
    ipc.answer = (cmd) =>
      cmd === "task_story"
        ? Promise.resolve({
            id: "01A",
            pages: [
              { n: 1, at: "2026-08-12T10:00:00Z", by: "dev_a", chapter: "opened" },
              { n: 2, at: "2026-08-13T10:00:00Z", by: "dev_a", chapter: "shut" },
            ],
          })
        : Promise.resolve(null);

    render(<Trail task="01A" lists={[]} />);
    await screen.findByRole("list");

    expect(screen.getByText(/let to an agent/i)).toBeTruthy();
    expect(screen.getByText(/kept to yourself again/i)).toBeTruthy();
  });

  // The conversion is the one chapter written while the detail is open: the trail asks again
  // when the task moved, not only when another task is opened.
  it("asks for the story again when the task moved under an open detail", async () => {
    ipc.answer = (cmd) =>
      cmd === "task_story" ? Promise.resolve({ id: "01B", pages: [] }) : Promise.resolve(null);
    const { rerender } = render(<Trail task="01B" moved="done||" lists={[]} />);
    await waitFor(() => expect(sent("task_story")).toHaveLength(1));

    rerender(<Trail task="01B" moved="done|trace|" lists={[]} />);
    await waitFor(() => expect(sent("task_story")).toHaveLength(2));

    rerender(<Trail task="01B" moved="done|trace|" lists={[]} />);
    expect(sent("task_story")).toHaveLength(2);
  });
});
