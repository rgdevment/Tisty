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
    knowAgents({ dev_agent: "canelo 39" });
    render(<TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

    expect(screen.getByText("by canelo 39")).toBeTruthy();
    expect(screen.getByText("#seguridad")).toBeTruthy();
    expect(screen.queryByText(/#agent/)).toBeNull();
  });

  it("keeps the tag, and signs nothing, when the writer is not a known agent", () => {
    render(<TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

    expect(screen.queryByText(/by /)).toBeNull();
    expect(screen.getByText("#agent #seguridad")).toBeTruthy();
  });

  it("signs the detail under the title while open, and in the stamps once closed", () => {
    knowAgents({ dev_agent: "canelo 39" });
    const { unmount } = detail(task());
    expect(screen.getByText("by canelo 39")).toBeTruthy();
    unmount();

    detail(task({ status: "done", completed_at: "2026-09-10T10:00:00Z" }));
    expect(screen.getByText(/· by canelo 39/)).toBeTruthy();
  });

  it("is read out loud too, apart from what the agent said about it", () => {
    knowAgents({ dev_agent: "canelo 39" });
    expect(spokenLabel(task())).toBe("renew the certificate — by canelo 39");
    expect(
      spokenLabel(
        task({ resolved: { by: "dev_agent", at: "2026-09-10T10:00:00Z", entry: "01E" } }),
      ),
    ).toBe("renew the certificate — by canelo 39 — canelo 39 says this is done");
  });
});
