import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { credit } from "../crediting";
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

const detail = (one: Task, expanded = true) =>
  render(
    <Detail
      task={one}
      expanded={expanded}
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
      onClose={() => {}}
    />,
  );

// At a glance a row only says an agent filed it; the name waits in the tooltip and in what is read
// aloud, and never comes from the tag the server adds, which the person cannot trust.
describe("a task an agent filed carries its signature", () => {
  afterEach(() => {
    knowAgents({});
    credit("rows", "agent");
    credit("inside", "named");
  });

  it("signs the row as an agent's, names it on hover, and drops the tag that said the same", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      { clients: { "claude-code": "Claude Code", codex: "Codex" } },
    );
    render(<TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

    expect(screen.getByText("by an agent").getAttribute("title")).toBe("Claude Code wrote it");
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

    expect(screen.getByText("by an agent").getAttribute("title")).toBe("An agent wrote it");
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

  it("signs the side panel as the full page does, so its title and its history agree", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      { clients: { "claude-code": "Claude Code", codex: "Codex" } },
    );
    const { unmount } = detail(task(), false);
    expect(screen.getByText("by Claude Code")).toBeTruthy();
    expect(screen.queryByText("by an agent")).toBeNull();
    unmount();

    credit("inside", "agent");
    detail(task(), false);
    expect(screen.getByText("by an agent").getAttribute("title")).toBe("Claude Code wrote it");
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

    expect(screen.getByText("by an agent").getAttribute("title")).toBe("Kiro wrote it");
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

  it("says on hover who wrote it and on which machine, and only that", () => {
    const wrote = (hosts: Record<string, string>) => {
      knowAgents(
        { dev_agent: "canelo 39" },
        {
          clients: { "claude-code": "Claude Code" },
          hosts,
          machines: { dev_desk: "roble 12", dev_lap: "sauce 4" },
          here: "dev_lap",
        },
      );
      const { unmount } = render(
        <TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />,
      );
      const said = screen.getByText("by an agent").getAttribute("title");
      unmount();
      return said;
    };

    expect(wrote({ dev_agent: "dev_lap" })).toBe("Claude Code wrote on this machine");
    expect(wrote({ dev_agent: "dev_desk" })).toBe("Claude Code wrote on the machine roble 12");
  });

  it("names the agent in the row when the person asks for it, with its machine on hover", () => {
    knowAgents(
      { dev_agent: "canelo 39" },
      {
        clients: { "claude-code": "Claude Code" },
        hosts: { dev_agent: "dev_lap" },
        here: "dev_lap",
      },
    );
    credit("rows", "named");
    render(<TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

    expect(screen.getByText("by Claude Code").getAttribute("title")).toBe("from this machine");
    expect(screen.queryByText("by an agent")).toBeNull();
  });

  it("says only an agent inside the task when asked, with the name left on hover", () => {
    knowAgents({ dev_agent: "canelo 39" }, { clients: { "claude-code": "Claude Code" } });
    credit("inside", "agent");
    const noted = task({
      log: [
        {
          id: "01E",
          at: "2026-11-02T11:46:00Z",
          body: "molde del menú",
          by: "dev_agent",
          via: "claude-code",
        },
      ],
      resolved: { by: "dev_agent", at: "2026-09-10T10:00:00Z", entry: "01E", via: "claude-code" },
    });
    detail(noted);

    expect(screen.queryByText(/Claude Code/)).toBeNull();
    const signs = screen.getAllByText("by an agent").map((one) => one.getAttribute("title"));
    expect(signs).toEqual(["Claude Code wrote it", "Claude Code wrote it"]);
    expect(screen.getByText("An agent says this is done")).toBeTruthy();
  });

  it("keeps the name out of a closed task's stamps when asked", () => {
    knowAgents({ dev_agent: "canelo 39" }, { clients: { "claude-code": "Claude Code" } });
    credit("inside", "agent");
    detail(
      task({
        status: "done",
        completed_at: "2026-09-10T10:00:00Z",
        resolved: { by: "dev_agent", at: "2026-09-10T10:00:00Z", entry: "01E", via: "claude-code" },
      }),
    );

    expect(screen.queryByText(/Claude Code/)).toBeNull();
    expect(screen.getByText(/· by an agent/).getAttribute("title")).toBe("Claude Code wrote it");
  });

  it("draws only a diamond everywhere when both places ask for it", () => {
    knowAgents({ dev_agent: "canelo 39" }, { clients: { "claude-code": "Claude Code" } });
    credit("rows", "mark");
    credit("inside", "mark");
    const { unmount } = render(
      <TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />,
    );
    expect(screen.queryByText(/by /)).toBeNull();
    expect(screen.getByRole("img", { name: "Claude Code wrote it" }).textContent).toBe("◆");
    unmount();

    detail(
      task({
        log: [
          {
            id: "01E",
            at: "2026-11-02T11:46:00Z",
            body: "notas",
            by: "dev_agent",
            via: "claude-code",
          },
        ],
        resolved: { by: "dev_agent", at: "2026-09-10T10:00:00Z", entry: "01E", via: "claude-code" },
      }),
    );
    expect(screen.queryByText(/by |Claude Code/)).toBeNull();
    expect(screen.getAllByRole("img", { name: "Claude Code wrote it" })).toHaveLength(2);
    expect(screen.getByText("An agent says this is done")).toBeTruthy();
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

describe("each place follows its own signature", () => {
  afterEach(() => {
    knowAgents({});
    credit("rows", "agent");
    credit("inside", "named");
  });

  const known = () =>
    knowAgents({ dev_agent: "canelo 39" }, { clients: { "claude-code": "Claude Code" } });
  const list = () =>
    render(<TaskList tasks={[task()]} lists={[]} title="Open" bands="day" onSelect={() => {}} />);

  it("names the agent in the list while the task's own page shows only a diamond", () => {
    known();
    credit("rows", "named");
    credit("inside", "mark");
    const { unmount } = list();
    expect(screen.getByText("by Claude Code")).toBeTruthy();
    unmount();

    detail(task());
    expect(screen.queryByText(/by /)).toBeNull();
    expect(screen.getByRole("img", { name: "Claude Code wrote it" }).textContent).toBe("◆");
  });

  it("draws a diamond in the list while the side panel names the agent", () => {
    known();
    credit("rows", "mark");
    credit("inside", "named");
    const { unmount } = list();
    expect(screen.queryByText(/by /)).toBeNull();
    expect(screen.getByRole("img", { name: "Claude Code wrote it" })).toBeTruthy();
    unmount();

    detail(task(), false);
    expect(screen.getByText("by Claude Code")).toBeTruthy();
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("says only an agent in the list while the side panel stays on the name", () => {
    known();
    credit("rows", "agent");
    credit("inside", "named");
    const { unmount } = list();
    expect(screen.getByText("by an agent")).toBeTruthy();
    unmount();

    detail(task(), false);
    expect(screen.getByText("by Claude Code")).toBeTruthy();
    expect(screen.queryByText("by an agent")).toBeNull();
  });
});

describe("what the agent said keeps its name on hover when the name is hidden", () => {
  afterEach(() => {
    knowAgents({});
    credit("inside", "named");
  });

  const resolved = {
    by: "dev_agent",
    at: "2026-09-10T10:00:00Z",
    entry: "01E",
    via: "claude-code",
  };
  const known = () =>
    knowAgents({ dev_agent: "canelo 39" }, { clients: { "claude-code": "Claude Code" } });
  const bannerOf = (text: string) => screen.getByText(text).closest("p") as HTMLElement;

  it("titles the open task's banner with the agent while it says only an agent", () => {
    known();
    credit("inside", "agent");
    detail(task({ resolved }));
    expect(bannerOf("An agent says this is done").getAttribute("title")).toBe(
      "Claude Code says this is done",
    );
  });

  it("titles the banner the same way under a diamond", () => {
    known();
    credit("inside", "mark");
    detail(task({ resolved }));
    expect(bannerOf("An agent says this is done").getAttribute("title")).toBe(
      "Claude Code says this is done",
    );
  });

  it("leaves the banner untitled when it already names the agent", () => {
    known();
    credit("inside", "named");
    detail(task({ resolved }));
    expect(bannerOf("Claude Code says this is done").hasAttribute("title")).toBe(false);
  });

  it("titles a closed task's stamp with the agent unless it is named", () => {
    known();
    credit("inside", "agent");
    const closed = task({ status: "done", completed_at: "2026-09-10T10:00:00Z", resolved });
    const { container, unmount } = detail(closed);
    const stamp = () => container.querySelector("span.text-hue-teal[title]");
    expect(stamp()?.getAttribute("title")).toMatch(/Claude Code/);
    unmount();

    credit("inside", "named");
    const named = detail(closed);
    expect(named.container.querySelector("span.text-hue-teal[title]")).toBeNull();
  });
});
