import { render, screen } from "@testing-library/react";
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
    title: "write the report",
    status: "open",
    priority: "do",
    order: "a0",
    steps: [],
    log: [],
    tags: [],
    ...extra,
  }) as unknown as Task;

const open = (one: Task, expanded = false) => {
  const done = vi.fn();
  render(
    <Detail
      task={one}
      lists={[]}
      known={[]}
      expanded={expanded}
      onExpand={() => {}}
      onCollapse={() => {}}
      onPatch={() => {}}
      onStep={() => {}}
      onMark={() => {}}
      onDropStep={() => {}}
      onLog={() => {}}
      onComplete={done}
      onDiscard={() => {}}
      onReopen={() => {}}
      onStillOpen={() => {}}
      onErase={() => {}}
      onClose={() => {}}
    />,
  );
  return done;
};

describe("finishing a task from the panel", () => {
  it("completes it from the column", async () => {
    const done = open(task());

    await userEvent.click(screen.getByRole("button", { name: /complete/i }));

    expect(done).toHaveBeenCalled();
  });

  it("completes it full-screen too", async () => {
    const done = open(task(), true);

    await userEvent.click(screen.getByRole("button", { name: /complete/i }));

    expect(done).toHaveBeenCalled();
  });

  it("keeps the way out of a task it will not do", () => {
    open(task());

    expect(screen.getByRole("button", { name: /not doing it/i })).toBeTruthy();
  });

  it("offers to reopen instead once the task is settled", () => {
    open(task({ status: "done" }));

    expect(screen.queryByRole("button", { name: /complete/i })).toBeNull();
    expect(screen.getByRole("button", { name: /reopen/i })).toBeTruthy();
  });
});

describe("what an agent says is done", () => {
  afterEach(() => knowAgents({}));

  const marked = { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1" };

  const shown = (one: Task) => {
    const back = vi.fn();
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
        onStillOpen={back}
        onErase={() => {}}
        onClose={() => {}}
      />,
    );
    return back;
  };

  it("is said on the task itself, not only in the list", () => {
    shown(task({ resolved: marked } as Partial<Task>));

    expect(screen.getByText("An agent says this is done")).toBeTruthy();
  });

  it("can be sent back without finishing it", async () => {
    const back = shown(task({ resolved: marked } as Partial<Task>));

    await userEvent.click(screen.getByRole("button", { name: /still to do/i }));

    expect(back).toHaveBeenCalled();
    expect(
      screen.getByRole("button", { name: /complete/i }),
      "finishing it is still there, and still the person's",
    ).toBeTruthy();
  });

  it("calls the agent by the name the person sees in settings", () => {
    knowAgents({ dev_agent: "peral 76" });
    shown(task({ resolved: { ...marked, by: "dev_agent" } } as Partial<Task>));

    expect(screen.getByText("peral 76 says this is done")).toBeTruthy();
  });

  it("falls back to saying it was an agent when the name is not known here", () => {
    knowAgents({});
    shown(task({ resolved: { ...marked, by: "dev_gone" } } as Partial<Task>));

    expect(screen.getByText("An agent says this is done")).toBeTruthy();
  });

  it("signs each journal entry an agent wrote, and leaves the person's unsigned", () => {
    knowAgents({ dev_agent: "peral 76" });
    shown(
      task({
        resolved: marked,
        log: [
          { id: "l1", at: "2026-08-06T18:40:00Z", body: "0 errores", by: "dev_agent" },
          { id: "l2", at: "2026-08-06T19:00:00Z", body: "lo apunto yo", by: "dev_laptop" },
        ],
      } as Partial<Task>),
    );

    expect(screen.getByText("by peral 76")).toBeTruthy();
    expect(screen.queryByText("by dev_laptop")).toBeNull();
  });

  it("says nothing on a task no agent spoke for", () => {
    shown(task());

    expect(screen.queryByText("An agent says this is done")).toBeNull();
    expect(screen.queryByRole("button", { name: /still to do/i })).toBeNull();
  });
});

describe("erasing what is already archived", () => {
  const shown = (one: Task, onErase = vi.fn()) => {
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
        onErase={onErase}
        onClose={() => {}}
      />,
    );
    return onErase;
  };

  it("is offered on a dropped task", async () => {
    const erased = shown(task({ status: "dropped" }));

    await userEvent.click(screen.getByRole("button", { name: /erase for good/i }));

    expect(erased).toHaveBeenCalled();
  });

  it("is offered on a completed task that was put away", () => {
    shown(task({ status: "done", hidden: true }));

    expect(screen.getByRole("button", { name: /erase for good/i })).toBeTruthy();
  });

  it("is not offered on a completed task still in plain sight", () => {
    shown(task({ status: "done" }));

    expect(screen.queryByRole("button", { name: /erase for good/i })).toBeNull();
  });

  it("is never offered while the task is open", () => {
    shown(task({ status: "open" }));

    expect(screen.queryByRole("button", { name: /erase for good/i })).toBeNull();
  });
});
