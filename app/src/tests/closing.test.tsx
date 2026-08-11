import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Closing from "../ui/Closing";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.calls = [];
});

const sent = () => ipc.calls.filter((one) => one.cmd === "close_window");

describe("the closing question", () => {
  it("hides the window and remembers when told to", async () => {
    render(<Closing onDismiss={() => {}} onError={() => {}} />);

    await userEvent.click(screen.getByRole("button", { name: /leave it in the tray/i }));

    await waitFor(() => expect(sent().length).toBe(1));
    expect(sent()[0].args).toMatchObject({ how: "hide", remember: true });
  });

  it("quits without remembering when the box is cleared", async () => {
    render(<Closing onDismiss={() => {}} onError={() => {}} />);

    await userEvent.click(screen.getByRole("checkbox"));
    await userEvent.click(screen.getByRole("button", { name: /close tisty/i }));

    await waitFor(() => expect(sent().length).toBe(1));
    expect(sent()[0].args).toMatchObject({ how: "quit", remember: false });
  });

  /// Backing out is not an answer: the question has to come again.
  it("decides nothing when it is dismissed", async () => {
    const dismissed = vi.fn();
    render(<Closing onDismiss={dismissed} onError={() => {}} />);

    await userEvent.click(screen.getByRole("button", { name: /stay open/i }));

    expect(dismissed).toHaveBeenCalled();
    expect(sent().length).toBe(0);
  });

  it("is a dialog, and Escape backs out of it", async () => {
    const dismissed = vi.fn();
    render(<Closing onDismiss={dismissed} onError={() => {}} />);

    expect(screen.getByRole("dialog").getAttribute("aria-modal")).toBe("true");
    await userEvent.keyboard("{Escape}");

    expect(dismissed).toHaveBeenCalled();
    expect(sent().length).toBe(0);
  });
});
