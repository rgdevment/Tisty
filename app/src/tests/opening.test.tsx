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
      expanded={false}
      onExpand={() => {}}
      onCollapse={() => {}}
      onPatch={() => {}}
      onStep={() => {}}
      onMark={() => {}}
      onDropStep={() => {}}
      onLog={() => {}}
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

  it("is offered on an open task the person wrote, behind «more», and opens it", async () => {
    const door = open(task());

    await userEvent.click(screen.getByRole("button", { name: /^more$/i }));
    await userEvent.click(screen.getByRole("menuitem", { name: /^allow agents$/i }));

    expect(door).toHaveBeenCalledWith(true);
    expect(screen.queryByText(/open to agents/i)).toBeNull();
  });

  it("says the task is open, and offers to keep it again", async () => {
    const door = open(task({ open_to_agents: true }));

    expect(screen.getByText(/open to agents/i)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^more$/i }));
    await userEvent.click(screen.getByRole("menuitem", { name: /^no agents$/i }));

    expect(door).toHaveBeenCalledWith(false);
  });

  it("is not offered on what an agent filed, which is the agents' already", () => {
    knowAgents({ dev_agent: "peral 76" });
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
