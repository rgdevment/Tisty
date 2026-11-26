import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt } from "../locales";
import NewHookAsk from "../ui/NewHookAsk";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answer: (_cmd: string): Promise<unknown> => Promise.resolve({ id: "doc-1", title: "guard.sh" }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answer(cmd);
  },
}));

const create = () => screen.getByRole("button", { name: "Create" }) as HTMLButtonElement;
const nameField = () => screen.getByLabelText("Name") as HTMLInputElement;

beforeEach(() => {
  adopt("en");
  ipc.calls.length = 0;
  ipc.answer = () => Promise.resolve({ id: "doc-1", title: "guard.sh" });
});

describe("the ask that makes a new hook", () => {
  it("keeps Create off until the name fits and says why when it does not", () => {
    render(<NewHookAsk connected={["codex"]} onMade={vi.fn()} onClose={vi.fn()} />);
    expect(create().disabled).toBe(true);
    fireEvent.change(nameField(), { target: { value: "bad name" } });
    expect(create().disabled).toBe(true);
    expect(screen.getByText(/^Use lowercase letters/)).toBeTruthy();
    fireEvent.change(nameField(), { target: { value: "-x" } });
    expect(create().disabled).toBe(true);
    fireEvent.change(nameField(), { target: { value: "no-rm" } });
    expect(create().disabled).toBe(false);
  });

  it("keeps Create off with no agent chosen or a timeout out of range", () => {
    render(<NewHookAsk connected={["codex"]} onMade={vi.fn()} onClose={vi.fn()} />);
    fireEvent.change(nameField(), { target: { value: "guard" } });
    expect(create().disabled).toBe(false);
    fireEvent.click(screen.getByLabelText("Codex"));
    expect(create().disabled).toBe(true);
    fireEvent.click(screen.getByLabelText("Codex"));
    const timeout = screen.getByLabelText("Seconds at most");
    fireEvent.change(timeout, { target: { value: "601" } });
    expect(create().disabled).toBe(true);
    fireEvent.change(timeout, { target: { value: "0" } });
    expect(create().disabled).toBe(true);
    fireEvent.change(timeout, { target: { value: "600" } });
    expect(create().disabled).toBe(false);
  });

  it("shows Tools only on a moment around a tool", () => {
    render(<NewHookAsk connected={[]} onMade={vi.fn()} onClose={vi.fn()} />);
    const when = screen.getByLabelText("When");
    expect(screen.queryByLabelText("Tools")).not.toBeNull();
    fireEvent.change(when, { target: { value: "after-tool" } });
    expect(screen.queryByLabelText("Tools")).not.toBeNull();
    for (const event of ["start", "prompt", "stop"]) {
      fireEvent.change(when, { target: { value: event } });
      expect(screen.queryByLabelText("Tools")).toBeNull();
    }
  });

  it("sends the name with its extension for this computer only by default", async () => {
    const onMade = vi.fn();
    render(<NewHookAsk connected={["claude-code", "codex"]} onMade={onMade} onClose={vi.fn()} />);
    fireEvent.change(nameField(), { target: { value: "guard" } });
    fireEvent.change(screen.getByLabelText("Script"), { target: { value: "sh" } });
    fireEvent.click(create());
    await waitFor(() => expect(onMade).toHaveBeenCalledWith({ id: "doc-1", title: "guard.sh" }));
    expect(ipc.calls).toEqual([
      {
        cmd: "agent_hook_new",
        args: {
          asked: {
            name: "guard.sh",
            event: "before-tool",
            tools: "shell",
            timeout: 10,
            agents: ["claude-code", "codex"],
            everyone: false,
          },
        },
      },
    ]);
  });

  it("sends tools any for a moment that has none and everyone when this computer only is off", async () => {
    render(<NewHookAsk connected={["cursor"]} onMade={vi.fn()} onClose={vi.fn()} />);
    fireEvent.change(nameField(), { target: { value: "done" } });
    fireEvent.change(screen.getByLabelText("Script"), { target: { value: "py" } });
    fireEvent.change(screen.getByLabelText("When"), { target: { value: "stop" } });
    fireEvent.click(screen.getByLabelText("Only this computer"));
    fireEvent.click(create());
    await waitFor(() => expect(ipc.calls).toHaveLength(1));
    expect(ipc.calls[0].args).toEqual({
      asked: {
        name: "done.py",
        event: "stop",
        tools: "any",
        timeout: 10,
        agents: ["cursor"],
        everyone: true,
      },
    });
  });

  it("says what was refused and stays open", async () => {
    ipc.answer = () => Promise.reject({ code: "agentHookNameTaken" });
    const onMade = vi.fn();
    render(<NewHookAsk connected={["codex"]} onMade={onMade} onClose={vi.fn()} />);
    fireEvent.change(nameField(), { target: { value: "guard" } });
    fireEvent.click(create());
    await waitFor(() => expect(screen.getByText("A hook already has that name")).toBeTruthy());
    expect(onMade).not.toHaveBeenCalled();
  });
});
