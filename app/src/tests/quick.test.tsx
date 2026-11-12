import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Quick from "../Quick";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
}));

const win = vi.hoisted(() => ({
  hidden: 0,
  focus: null as ((state: { payload: boolean }) => void) | null,
  emitted: [] as string[],
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
  listen: () => Promise.resolve(() => {}),
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

  it("spends its height on the sentence, not on instructions", async () => {
    render(<Quick />);
    await screen.findByRole("textbox");

    expect(screen.queryByText(/Esc/)).toBeNull();
    expect(screen.queryByText(/Ctrl\+Shift\+Space/)).toBeNull();
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
});
