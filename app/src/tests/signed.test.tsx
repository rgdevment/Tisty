import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import Detail from "../ui/Detail";
import { spokenLabel } from "../ui/Spoke";
import TaskList from "../ui/TaskList";
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
    tags: ["agent", "seguridad"],
    created_by: "dev_agent",
    created_via: "claude-code",
    ...extra,
  }) as unknown as Task;

const detail = (one: Task) =>
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
      onOpenToAgents={() => {}}
      onClose={() => {}}
    />,
  );

// Who filed a task is said the way the journal says who wrote a line — by name — and never
// through the tag the server adds, which the person cannot trust.
describe("a task an agent filed carries its signature", () => {
  afterEach(() => knowAgents({}));

  it("signs the row by name and drops the tag that said the same", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      { clients: { "claude-code": "Claude Code", codex: "Codex" } },
    );
    render(<TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

    expect(screen.getByText("by Claude Code")).toBeTruthy();
    expect(screen.getByText("#seguridad")).toBeTruthy();
    expect(screen.queryByText(/#agent/)).toBeNull();
    expect(screen.queryByText(/canelo 39/)).toBeNull();
  });

  it("calls what was written before clients had names the work of an assistant", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      { clients: { "claude-code": "Claude Code", codex: "Codex" } },
    );
    render(
      <TaskList
        tasks={[task({ created_via: undefined })]}
        lists={[]}
        title="Open"
        bands="day"
        onSelect={() => {}}
      />,
    );

    expect(screen.getByText("by an assistant")).toBeTruthy();
  });

  it("keeps the tag, and signs nothing, when the writer is not a known agent", () => {
    render(<TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

    expect(screen.queryByText(/by /)).toBeNull();
    expect(screen.getByText("#agent #seguridad")).toBeTruthy();
  });

  it("signs the detail under the title while open, and in the stamps once closed", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      { clients: { "claude-code": "Claude Code", codex: "Codex" } },
    );
    const { unmount } = detail(task());
    expect(screen.getByText("by Claude Code")).toBeTruthy();
    unmount();

    detail(task({ status: "done", completed_at: "2026-09-10T10:00:00Z" }));
    expect(screen.getByText(/· by Claude Code/)).toBeTruthy();
  });

  it("reads a client the core did not name as it called itself, never as nothing", () => {
    knowAgents({ dev_agent: "canelo 39" });
    render(
      <TaskList
        tasks={[task({ created_via: "kiro" })]}
        lists={[]}
        title="Open"
        bands="day"
        onSelect={() => {}}
      />,
    );

    expect(screen.getByText("by kiro")).toBeTruthy();
  });

  // The tooltip names machines the way Settings › Sync does, and never by their id.
  it("says which machine the hand wrote from, by the name the person knows it by", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      {
        clients: { "claude-code": "Claude Code" },
        hosts: { dev_agent: "dev_desk" },
        machines: { dev_desk: "roble 12", dev_lap: "sauce 4" },
        here: "dev_lap",
      },
    );
    const { unmount } = detail(task());
    expect(screen.getByText("by Claude Code").getAttribute("title")).toBe("from roble 12");
    unmount();

    knowAgents(
      { dev_agent: "canelo 39" },
      {
        clients: { "claude-code": "Claude Code" },
        hosts: { dev_agent: "dev_lap" },
        machines: { dev_lap: "sauce 4" },
        here: "dev_lap",
      },
    );
    detail(task());
    expect(screen.getByText("by Claude Code").getAttribute("title")).toBe("from this machine");
  });

  it("is read out loud too, apart from what the agent said about it", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      { clients: { "claude-code": "Claude Code", codex: "Codex" } },
    );
    expect(spokenLabel(task())).toBe("renew the certificate — by Claude Code");
    expect(
      spokenLabel(
        task({
          resolved: { by: "dev_agent", at: "2026-09-10T10:00:00Z", entry: "01E", via: "codex" },
        }),
      ),
    ).toBe("renew the certificate — by Claude Code — Codex says this is done");
  });
});
