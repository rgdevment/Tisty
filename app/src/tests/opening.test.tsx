import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";
import { knowAgents } from "../who";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const task = (extra: Partial<Task> = {}): Task =>
  ({
    id: "01T",
    title: "renew the certificate",
    status: "open",
    priority: "unset",
    order: "a0",
    steps: [],
    log: [],
    tags: [],
    created_by: "dev_laptop",
    ...extra,
  }) as unknown as Task;

const open = (one: Task) => {
  const door = vi.fn();
  render(
    <Detail
      task={one}
      lists={[]}
      known={[]}
      onCollapse={() => {}}
      onPatch={() => {}}
      onComplete={() => {}}
      onDiscard={() => {}}
      onReopen={() => {}}
      onStillOpen={() => {}}
      onErase={() => {}}
      onFold={() => {}}
      onReadAs={() => {}}
      onOpenToAgents={door}
      onClose={() => {}}
    />,
  );
  return door;
};

describe("letting an agent fill a task in", () => {
  afterEach(() => {
    cleanup();
    knowAgents({});
  });

  it("is no longer offered on a task the person wrote, which assistants may only speak for", async () => {
    open(task());

    const more = screen.queryByRole("button", { name: /^more$/i });
    if (more) await userEvent.click(more);
    expect(screen.queryByRole("menuitem", { name: /^allow agents$/i })).toBeNull();
  });

  it("says the task is open, and offers to keep it again", async () => {
    const door = open(task({ open_to_agents: true }));

    expect(screen.getByText(/open to agents/i)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^more$/i }));
    await userEvent.click(screen.getByRole("menuitem", { name: /^no agents$/i }));

    expect(door).toHaveBeenCalledWith(false);
  });

  it("is not offered on what an agent filed, which is the agents' already", () => {
    knowAgents(
      { dev_agent: "peral 76" },
      { clients: { "claude-code": "Claude Code", codex: "Codex" } },
    );
    open(task({ created_by: "dev_agent" }));

    expect(screen.queryByRole("button", { name: /^more$/i })).toBeNull();
  });

  it("is not offered once the task is closed, and the closing says it was open", () => {
    open(task({ status: "done", completed_at: "2026-09-10T10:00:00Z", open_to_agents: true }));

    expect(screen.queryByRole("menuitem", { name: /^no agents$/i })).toBeNull();
    expect(screen.queryByRole("menuitem", { name: /^allow agents$/i })).toBeNull();
    expect(screen.getByText(/· was open to agents/i)).toBeTruthy();
  });
});

describe("steps an assistant says are done", () => {
  it("wait on the task's page until the person ticks them or says not yet", async () => {
    const answer = vi.fn();
    render(
      <Detail
        task={task({
          steps: [
            { id: "s1", text: "pay the fee", done: false, order: "a0", said: true },
            { id: "s2", text: "download it", done: false, order: "a1" },
          ],
        } as Partial<Task>)}
        lists={[]}
        known={[]}
        onCollapse={() => {}}
        onPatch={() => {}}
        onComplete={() => {}}
        onDiscard={() => {}}
        onReopen={() => {}}
        onStillOpen={() => {}}
        onStepsSaid={answer}
        onErase={() => {}}
        onFold={() => {}}
        onReadAs={() => {}}
        onOpenToAgents={() => {}}
        onClose={() => {}}
      />,
    );

    expect(screen.getByText("An assistant says this step is done")).toBeTruthy();
    expect(screen.getByText("pay the fee")).toBeTruthy();
    expect(screen.queryByText("download it")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Tick them" }));

    expect(answer).toHaveBeenCalledWith(true);
  });
});
