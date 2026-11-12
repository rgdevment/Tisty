import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Quick from "../Quick";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
}));

const win = vi.hoisted(() => ({
  hidden: 0,
  focus: null as ((state: { payload: boolean }) => void) | null,
  emitted: [] as string[],
  heard: {} as Record<string, (event: { payload: unknown }) => void>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    switch (cmd) {
      case "snapshot":
        return Promise.resolve({
          tasks: [],
          lists: [],
          tags: [],
          refs: [],
          counts: {},
          locale: "en",
        });
      case "read":
        return Promise.resolve({
          title: String(args?.text ?? ""),
          tags: [],
          spans: [],
          offers: [],
        });
      case "jot":
        return Promise.resolve({ file: "mac0-0001", made: true });
      case "shortcut":
        return Promise.resolve("Ctrl+Shift+Space");
      case "capture":
        return Promise.resolve({
          id: "01T",
          title: String(args?.text ?? "milk"),
          status: "open",
          order: "a0",
          steps: [],
          journal: [],
          tags: [],
          created: "2026-08-10T10:00:00Z",
        });
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "quick",
    hide: () => {
      win.hidden += 1;
      return Promise.resolve();
    },
    onFocusChanged: (fn: (state: { payload: boolean }) => void) => {
      win.focus = fn;
      return Promise.resolve(() => {});
    },
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  emit: (name: string) => {
    win.emitted.push(name);
    return Promise.resolve();
  },
  listen: (name: string, fn: (event: { payload: unknown }) => void) => {
    win.heard[name] = fn;
    return Promise.resolve(() => {});
  },
}));

beforeEach(() => {
  ipc.calls = [];
  win.hidden = 0;
  win.focus = null;
  win.emitted = [];
});

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

describe("capturing without the window", () => {
  it("files what was typed and tells the main window", async () => {
    render(<Quick />);
    const field = await screen.findByRole("textbox");

    await userEvent.type(field, "buy milk{Enter}");

    await waitFor(() => expect(sent("capture").length).toBe(1));
    expect(win.emitted).toContain("captured");
    expect(await screen.findByText(/buy milk/)).toBeTruthy();
  });

  it("goes away when it loses the focus", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");
    win.hidden = 0;

    win.focus?.({ payload: false });

    await waitFor(() => expect(win.hidden).toBeGreaterThan(0));
  });

  it("goes away on Escape", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");
    win.hidden = 0;

    await userEvent.keyboard("{Escape}");

    await waitFor(() => expect(win.hidden).toBeGreaterThan(0));
  });

  it("reads again every time it comes back", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");
    expect(sent("snapshot").length).toBe(1);

    win.focus?.({ payload: true });

    await waitFor(() => expect(sent("snapshot").length).toBe(2));
  });

  it("spends its height on the sentence, and its shortcut sits in the row of modes", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");

    expect(screen.queryByText(/Esc/)).toBeNull();
    const keys = await screen.findByText("Ctrl+Shift+Space");
    expect(keys.parentElement).toBe(screen.getByRole("button", { name: "Task" }).parentElement);
  });

  it("winks instead of giving an example when its turn in the hundred comes", async () => {
    localStorage.setItem("tisty.wink", JSON.stringify({ seen: 0, at: 1 }));
    render(<Quick />);
    await screen.findByRole("textbox");

    act(() => win.focus?.({ payload: true }));

    await waitFor(() =>
      expect(screen.getByRole("textbox").getAttribute("placeholder")).toBe(t("taskWink")),
    );
    localStorage.clear();
  });

  it("spends no turn of the hundred while it is only made, hidden, at start", async () => {
    localStorage.setItem("tisty.wink", JSON.stringify({ seen: 0, at: 1 }));
    render(<Quick />);
    await screen.findByRole("textbox");

    expect(JSON.parse(localStorage.getItem("tisty.wink") ?? "{}").seen).toBe(0);
    localStorage.clear();
  });

  it("invites a task with one of its examples each time it opens", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");

    act(() => win.focus?.({ payload: true }));

    await waitFor(() =>
      expect(t("taskExamples").split("\n")).toContain(
        screen.getByRole("textbox").getAttribute("placeholder"),
      ),
    );
  });

  it("holds an idea to 140 characters, a task to none", async () => {
    render(<Quick />);
    const task = await screen.findByRole("textbox");
    expect(task.getAttribute("maxlength")).toBeNull();

    await userEvent.keyboard("{Tab}");

    expect(screen.getByRole("textbox").getAttribute("maxlength")).toBe("140");
  });

  it("drops a loose idea in the brainstorm instead of filing a task, after Tab", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");

    await userEvent.keyboard("{Tab}");
    await userEvent.type(screen.getByRole("textbox"), "call the vet{Enter}");

    await waitFor(() => expect(sent("jot").length).toBe(1));
    expect(sent("jot")[0].args).toEqual({ idea: "call the vet", title: "Brainstorm" });
    expect(sent("capture")).toEqual([]);
    expect(win.emitted).toContain("jotted");
    expect(await screen.findByText(/call the vet/)).toBeTruthy();
  });

  it("goes back to a task when Tab is pressed again", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");

    await userEvent.keyboard("{Tab}{Tab}");
    await userEvent.type(screen.getByRole("textbox"), "buy milk{Enter}");

    await waitFor(() => expect(sent("capture").length).toBe(1));
    expect(sent("jot")).toEqual([]);
  });

  it("opens ready for an idea from the idea shortcut, and the same shortcut again closes it", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");

    act(() => win.heard.door?.({ payload: { door: "idea", again: false } }));
    expect(await screen.findByRole("button", { name: "Idea", pressed: true })).toBeTruthy();
    win.hidden = 0;

    act(() => win.heard.door?.({ payload: { door: "idea", again: true } }));
    await waitFor(() => expect(win.hidden).toBeGreaterThan(0));
  });

  it("switches to a task, without closing, when the task shortcut comes while an idea is open", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");
    act(() => win.heard.door?.({ payload: { door: "idea", again: false } }));
    await screen.findByRole("button", { name: "Idea", pressed: true });
    win.hidden = 0;

    act(() => win.heard.door?.({ payload: { door: "task", again: true } }));

    expect(await screen.findByRole("button", { name: "Task", pressed: true })).toBeTruthy();
    expect(win.hidden).toBe(0);
  });
});
