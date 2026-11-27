import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Task, Whole } from "../core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (id: string, more: Partial<Task> = {}): Task =>
  ({
    id,
    title: `task ${id}`,
    status: "open",
    priority: "unset",
    order: "a0",
    tags: [],
    reminders: [],
    log: [],
    created_by: "me",
    ...more,
  }) as unknown as Task;

const byAgent = (id: string, more: Partial<Task> = {}) =>
  task(id, { created_by: "dev_agent", created_via: "claude-code", ...more });

const claim = (more: Record<string, unknown> = {}) => ({
  at: "2026-08-06T18:40:00Z",
  by: "dev_agent",
  entry: "e1",
  via: "claude-code",
  ...more,
});

const load = async () => {
  const who = await import("../who");
  const locales = await import("../locales");
  const list = await import("../ui/TaskList");
  who.knowAgents({ dev_agent: "canelo 39" }, { clients: { "claude-code": "Claude Code" } });
  return { TaskList: list.default, t: locales.t, fill: locales.fill };
};

type Shown = Awaited<ReturnType<typeof load>>;

const show = (
  { TaskList }: Shown,
  tasks: Task[],
  props: Partial<React.ComponentProps<typeof TaskList>> = {},
) => render(<TaskList tasks={tasks} lists={[]} title="Open" onSelect={() => {}} {...props} />);

const heads = (container: HTMLElement) =>
  Array.from(container.querySelectorAll<HTMLElement>("[data-owner]"));

const order = (container: HTMLElement) =>
  Array.from(container.querySelectorAll<HTMLElement>("[data-task]")).map((row) => row.dataset.task);

beforeEach(() => {
  vi.resetModules();
  localStorage.clear();
});

afterEach(() => localStorage.clear());

describe("a list with only the person's tasks", () => {
  it("has no sections at all", async () => {
    const all = await load();
    const { container } = show(all, [task("1"), task("2")]);

    expect(heads(container)).toHaveLength(0);
    expect(screen.queryByText(all.t("personOwns"))).toBeNull();
    expect(screen.queryByText(all.t("agentOwns"))).toBeNull();
  });

  it("keeps the complete checkbox and no diamond", async () => {
    const all = await load();
    show(all, [task("1")], { onComplete: vi.fn() });

    expect(screen.getByRole("button", { name: all.fill("completeIt", "task 1") })).toBeTruthy();
    expect(screen.queryByText("◇")).toBeNull();
  });

  it("does not say the agent's word on a person's row, even with both answers wired", async () => {
    const all = await load();
    show(all, [task("1", { resolved: claim() })], {
      onConfirm: vi.fn(),
      onKeepOpen: vi.fn(),
      onConfirmAll: vi.fn(),
    });

    expect(screen.queryByText(all.t("markDone"))).toBeNull();
  });
});

describe("a list with a task the agent does", () => {
  it("puts the person's section first and the agent's after it, with a count each", async () => {
    const all = await load();
    const { container } = show(all, [
      byAgent("a1"),
      task("p1"),
      byAgent("a2"),
      task("p2"),
      task("p3"),
    ]);

    const [first, second] = heads(container);

    expect(heads(container)).toHaveLength(2);
    expect(first.dataset.owner).toBe("person");
    expect(within(first).getByText(all.t("personOwns"))).toBeTruthy();
    expect(within(first).getByText("3")).toBeTruthy();
    expect(second.dataset.owner).toBe("agent");
    expect(within(second).getByText(all.t("agentOwns"))).toBeTruthy();
    expect(within(second).getByText("2")).toBeTruthy();
    expect(order(container)).toEqual(["p1", "p2", "p3", "a1", "a2"]);
  });

  it("shows only the agent's section when every task is the agent's", async () => {
    const all = await load();
    const { container } = show(all, [byAgent("a1"), byAgent("a2")]);

    expect(heads(container).map((one) => one.dataset.owner)).toEqual(["agent"]);
  });

  it("gives a task the agent filed for the person to the person's section", async () => {
    const all = await load();
    const { container } = show(all, [byAgent("a1"), byAgent("m1", { for_person: true })]);

    expect(order(container)).toEqual(["m1", "a1"]);
    expect(within(heads(container)[0]).getByText("1")).toBeTruthy();
  });

  it("explains each section the first time, and not on the next mount", async () => {
    const all = await load();
    const first = show(all, [byAgent("a1"), task("p1")]);

    expect(screen.getByText(all.t("personOwnsWhy"))).toBeTruthy();
    expect(screen.getByText(all.t("agentOwnsWhy"))).toBeTruthy();
    first.unmount();

    show(all, [byAgent("a1"), task("p1")]);

    expect(screen.queryByText(all.t("personOwnsWhy"))).toBeNull();
    expect(screen.queryByText(all.t("agentOwnsWhy"))).toBeNull();
    expect(screen.getByText(all.t("personOwns"))).toBeTruthy();
  });

  it("keeps the explanation while the same list is redrawn", async () => {
    const all = await load();
    const view = show(all, [byAgent("a1"), task("p1")]);

    view.rerender(
      <all.TaskList
        tasks={[byAgent("a1"), task("p1"), task("p2")]}
        lists={[]}
        title="Open"
        onSelect={() => {}}
      />,
    );

    expect(screen.getByText(all.t("agentOwnsWhy"))).toBeTruthy();
  });

  it("does not split a dense list", async () => {
    const all = await load();
    const { container } = show(
      all,
      [byAgent("a1", { status: "done" }), task("p1", { status: "done" })],
      {
        dense: true,
      },
    );

    expect(heads(container)).toHaveLength(0);
  });

  it("marks an open agent row with a diamond in place of the checkbox", async () => {
    const all = await load();
    show(all, [byAgent("a1"), task("p1")], { onComplete: vi.fn() });

    expect(screen.getAllByTitle(all.t("agentDoesIt"))).toHaveLength(1);
    expect(screen.getByText("◇")).toBeTruthy();
    expect(screen.queryByRole("button", { name: all.fill("completeIt", "task a1") })).toBeNull();
    expect(screen.getByRole("button", { name: all.fill("completeIt", "task p1") })).toBeTruthy();
  });

  it("shows the agent's word on its rows without confirming mode, and not on the person's", async () => {
    const all = await load();
    const handlers = { onConfirm: vi.fn(), onKeepOpen: vi.fn() };
    const { container } = show(
      all,
      [byAgent("a1", { resolved: claim() }), task("p1", { resolved: claim({ by: "me" }) })],
      handlers,
    );

    const agent = container.querySelector<HTMLElement>('[data-task="a1"]');
    const person = container.querySelector<HTMLElement>('[data-task="p1"]');

    expect(within(agent as HTMLElement).getByText(all.t("markDone"))).toBeTruthy();
    expect(within(person as HTMLElement).queryByText(all.t("markDone"))).toBeNull();
  });

  it("shows the word on person rows too when confirming", async () => {
    const all = await load();
    const { container } = show(
      all,
      [byAgent("a1"), task("p1", { resolved: claim({ by: "me" }) })],
      { onConfirm: vi.fn(), onKeepOpen: vi.fn(), confirming: true },
    );

    const person = container.querySelector<HTMLElement>('[data-task="p1"]');

    expect(within(person as HTMLElement).getByText(all.t("markDone"))).toBeTruthy();
  });

  it("does not show the word on agent rows unless both answers are wired", async () => {
    const all = await load();
    show(all, [byAgent("a1", { resolved: claim() })], { onConfirm: vi.fn() });

    expect(screen.queryByText(all.t("markDone"))).toBeNull();
  });

  it("calls back with the task the person confirms", async () => {
    const all = await load();
    const onConfirm = vi.fn();
    const one = byAgent("a1", { resolved: claim() });
    show(all, [one], { onConfirm, onKeepOpen: vi.fn() });

    await userEvent.click(screen.getByRole("button", { name: new RegExp(all.t("markDone")) }));

    expect(onConfirm).toHaveBeenCalledWith(one);
  });

  it("counts every row in the group the way the rows go, band by band", async () => {
    const all = await load();
    const day = (at: string) => ({
      at: `${at}T10:00:00`,
      tz: "UTC",
      floating: true,
      has_time: false,
    });
    const { container } = show(
      all,
      [
        byAgent("a1", { date: day("2026-08-11") }),
        byAgent("a2", { date: day("2026-08-12") }),
        task("p1", { date: day("2026-08-11") }),
      ],
      { bands: "month" },
    );

    expect(within(heads(container)[0]).getByText("1")).toBeTruthy();
    expect(within(heads(container)[1]).getByText("2")).toBeTruthy();
  });
});

describe("confirming everything shown", () => {
  const ready = (id: string, more: Partial<Task> = {}) =>
    byAgent(id, { resolved: claim(), ...more });

  it("is offered from two confirmable tasks, in the agent's section", async () => {
    const all = await load();
    const { container } = show(all, [ready("a1"), ready("a2"), task("p1")], {
      onConfirmAll: vi.fn(),
    });

    const agent = heads(container).find((one) => one.dataset.owner === "agent") as HTMLElement;
    const person = heads(container).find((one) => one.dataset.owner === "person") as HTMLElement;

    expect(within(agent).getByRole("button", { name: /2/ }).textContent).toContain(
      all.fill("confirmAllShown", "2"),
    );
    expect(within(person).queryByRole("button")).toBeNull();
  });

  it("is not offered for one confirmable task", async () => {
    const all = await load();
    const { container } = show(all, [ready("a1"), byAgent("a2")], { onConfirmAll: vi.fn() });

    expect(within(heads(container)[0]).queryByRole("button")).toBeNull();
  });

  it("is not offered without the callback", async () => {
    const all = await load();
    show(all, [ready("a1"), ready("a2")]);

    expect(screen.queryByText(new RegExp(all.fill("confirmAllShown", "2")))).toBeNull();
  });

  it("calls back with exactly the confirmable tasks of that group", async () => {
    const all = await load();
    const onConfirmAll = vi.fn();
    const ones = [ready("a1"), ready("a2"), ready("a3")];
    show(
      all,
      [
        ...ones,
        byAgent("open1"),
        ready("dropped1", { resolved: claim({ drop: true }) }),
        ready("again", {
          repeat: { each: { every: 1, unit: "week" } } as unknown as Task["repeat"],
        }),
        task("p1", { resolved: claim({ by: "me" }) }),
      ],
      { onConfirmAll },
    );

    await userEvent.click(
      screen.getByRole("button", { name: new RegExp(all.fill("confirmAllShown", "3")) }),
    );

    expect(onConfirmAll).toHaveBeenCalledTimes(1);
    expect(onConfirmAll.mock.calls[0][0]).toEqual(ones);
  });

  const mine = () => [
    task("p1", { resolved: claim({ by: "me" }) }),
    task("p2", { resolved: claim({ by: "me" }) }),
  ];

  it("is not offered in the person's section outside confirming", async () => {
    const all = await load();
    const { container } = show(all, [byAgent("a1"), ...mine()], { onConfirmAll: vi.fn() });

    const person = heads(container).find((one) => one.dataset.owner === "person") as HTMLElement;

    expect(within(person).queryByRole("button")).toBeNull();
  });

  it("is offered in the person's section when confirming, with exactly its confirmable tasks", async () => {
    const all = await load();
    const onConfirmAll = vi.fn();
    const ones = mine();
    const { container } = show(all, [byAgent("a1"), ...ones], { onConfirmAll, confirming: true });

    const person = heads(container).find((one) => one.dataset.owner === "person") as HTMLElement;
    await userEvent.click(within(person).getByRole("button"));

    expect(onConfirmAll).toHaveBeenCalledWith(ones);
  });

  it("still offers the agent's section when confirming is off", async () => {
    const all = await load();
    const { container } = show(all, [ready("a1"), ready("a2")], { onConfirmAll: vi.fn() });

    expect(within(heads(container)[0]).getByRole("button")).toBeTruthy();
  });

  describe("with a folded band", () => {
    beforeEach(() => vi.setSystemTime(new Date("2026-10-27T09:00:00")));
    afterEach(() => vi.useRealTimers());

    const spec = (day: string) => ({
      at: `${day}T10:00:00`,
      tz: "America/Santiago",
      floating: true,
      has_time: false,
    });
    const underway = (id: string) =>
      ready(id, { date: spec("2026-10-06"), deadline: spec("2026-12-04") } as Partial<Task>);

    it("neither counts nor passes a confirmable task inside it", async () => {
      const all = await load();
      const onConfirmAll = vi.fn();
      const kept = [ready("a1"), ready("a2")];
      const { container } = show(all, [...kept, underway("u1")], {
        onConfirmAll,
        bands: "day",
        standings: { u1: "underway" },
      });

      expect(screen.getByRole("button", { name: /Underway/ }).getAttribute("aria-expanded")).toBe(
        "false",
      );
      const button = within(heads(container)[0]).getByRole("button", {
        name: new RegExp(all.fill("confirmAllShown", "2")),
      });
      await userEvent.click(button);

      expect(onConfirmAll.mock.calls[0][0]).toEqual(kept);
    });

    it("offers nothing when only one confirmable task is outside the folded band", async () => {
      const all = await load();
      const { container } = show(all, [ready("a1"), underway("u1"), underway("u2")], {
        onConfirmAll: vi.fn(),
        bands: "day",
        standings: { u1: "underway", u2: "underway" },
      });

      expect(within(heads(container)[0]).queryByRole("button")).toBeNull();
    });

    it("counts the task once the folded band is opened", async () => {
      const all = await load();
      const onConfirmAll = vi.fn();
      const { container } = show(all, [ready("a1"), underway("u1")], {
        onConfirmAll,
        bands: "day",
        standings: { u1: "underway" },
      });

      await userEvent.click(screen.getByRole("button", { name: /Underway/ }));
      await userEvent.click(
        within(heads(container)[0]).getByRole("button", {
          name: new RegExp(all.fill("confirmAllShown", "2")),
        }),
      );

      expect(onConfirmAll.mock.calls[0][0].map((one: Task) => one.id).sort()).toEqual(["a1", "u1"]);
    });
  });
});

describe("a part meant for the person", () => {
  const wholes = (working: boolean): Record<string, Whole> => ({
    w1: { title: "the release", open: 2, closed: 0, working },
  });
  const part = () => byAgent("m1", { for_person: true, part_of: "w1" });

  it("says it waits on the whole while the agent works on it", async () => {
    const all = await load();
    show(all, [part()], { wholes: wholes(true) });

    expect(screen.getByText(`↳ ${all.fill("waitsOn", "the release")}`)).toBeTruthy();
    expect(screen.queryByText(/⌂/)).toBeNull();
  });

  it("says it is the person's part when the agent is not working on the whole", async () => {
    const all = await load();
    show(all, [part()], { wholes: wholes(false) });

    expect(screen.getByText(`↳ ${all.fill("yourPartOf", "the release")}`)).toBeTruthy();
  });

  it("keeps the whole chip for a part not meant for the person", async () => {
    const all = await load();
    show(all, [task("p1", { part_of: "w1" })], { wholes: wholes(true) });

    expect(screen.getByText("⌂ the release")).toBeTruthy();
    expect(screen.queryByText(/↳/)).toBeNull();
  });
});

describe("the agent's signature in the section", () => {
  const sign = (rows: string) => localStorage.setItem("tisty.credit", JSON.stringify({ rows }));
  const filed = () => [byAgent("a1"), task("p1")];

  it("signs the row as an agent's by default", async () => {
    const all = await load();
    show(all, filed());

    expect(screen.getByText("by an agent")).toBeTruthy();
  });

  it("names the client when the signing is named", async () => {
    sign("named");
    const all = await load();
    show(all, filed());

    expect(screen.getByText("by Claude Code")).toBeTruthy();
  });

  it("signs with a mark when the signing is a mark, and the proof word follows it", async () => {
    sign("mark");
    const all = await load();
    show(all, [byAgent("a1", { resolved: claim() })], { onConfirm: vi.fn(), onKeepOpen: vi.fn() });

    expect(screen.getByText(all.t("saidMarkDone")).textContent).toBe(`◆ ${all.t("saidMarkDone")}`);
  });

  it("words the claim anonymously for an agent signing", async () => {
    sign("agent");
    const all = await load();
    show(all, [byAgent("a1", { resolved: claim() })], { onConfirm: vi.fn(), onKeepOpen: vi.fn() });

    expect(screen.getByText(all.t("agentSaidDone"))).toBeTruthy();
  });

  it("words the claim with the client's name for a named signing", async () => {
    sign("named");
    const all = await load();
    show(all, [byAgent("a1", { resolved: claim() })], { onConfirm: vi.fn(), onKeepOpen: vi.fn() });

    expect(screen.getByText(all.fill("agentNamedSaidDone", "Claude Code"))).toBeTruthy();
  });
});

describe("the keyboard on the rows of a split list", () => {
  const rowOf = (container: HTMLElement, id: string) =>
    container.querySelector<HTMLElement>(`[data-task="${id}"]`) as HTMLElement;

  const press = async (row: HTMLElement) => {
    row.focus();
    await userEvent.keyboard("{Control>}{Enter}{/Control}");
  };

  const wired = () => ({
    onComplete: vi.fn(),
    onConfirm: vi.fn(),
    onKeepOpen: vi.fn(),
    onSelect: vi.fn(),
  });

  it("confirms an agent row that said it was done, and does not complete it", async () => {
    const all = await load();
    const handlers = wired();
    const one = byAgent("a1", { resolved: claim() });
    const { container } = show(all, [one, task("p1")], handlers);

    await press(rowOf(container, "a1"));

    expect(handlers.onConfirm).toHaveBeenCalledWith(one);
    expect(handlers.onComplete).not.toHaveBeenCalled();
    expect(handlers.onSelect).not.toHaveBeenCalled();
  });

  it("opens an agent row that said it would not do it", async () => {
    const all = await load();
    const handlers = wired();
    const one = byAgent("a1", { resolved: claim({ drop: true }) });
    const { container } = show(all, [one, task("p1")], handlers);

    await press(rowOf(container, "a1"));

    expect(handlers.onSelect).toHaveBeenCalledWith("a1", one);
    expect(handlers.onConfirm).not.toHaveBeenCalled();
    expect(handlers.onComplete).not.toHaveBeenCalled();
  });

  it("does nothing on an agent row that has said nothing", async () => {
    const all = await load();
    const handlers = wired();
    const { container } = show(all, [byAgent("a1"), task("p1")], handlers);

    await press(rowOf(container, "a1"));

    expect(handlers.onConfirm).not.toHaveBeenCalled();
    expect(handlers.onComplete).not.toHaveBeenCalled();
    expect(handlers.onSelect).not.toHaveBeenCalled();
  });

  it("does nothing on a said-done agent row when confirming is not wired", async () => {
    const all = await load();
    const handlers = wired();
    const { container } = show(all, [byAgent("a1", { resolved: claim() }), task("p1")], {
      ...handlers,
      onConfirm: undefined,
    });

    await press(rowOf(container, "a1"));

    expect(handlers.onComplete).not.toHaveBeenCalled();
  });

  it("still completes a person row", async () => {
    const all = await load();
    const handlers = wired();
    const { container } = show(all, [byAgent("a1"), task("p1")], handlers);

    await press(rowOf(container, "p1"));

    expect(handlers.onComplete).toHaveBeenCalledWith("p1", "task p1");
    expect(handlers.onConfirm).not.toHaveBeenCalled();
  });

  it("completes a row as before in a list that is not split", async () => {
    const all = await load();
    const handlers = wired();
    const { container } = show(all, [task("p1")], handlers);

    await press(rowOf(container, "p1"));

    expect(handlers.onComplete).toHaveBeenCalledWith("p1", "task p1");
  });

  it("announces the shortcut on an agent row only when it can confirm", async () => {
    const all = await load();
    const { container } = show(
      all,
      [byAgent("a1", { resolved: claim() }), byAgent("a2"), task("p1")],
      wired(),
    );

    expect(rowOf(container, "a1").getAttribute("aria-keyshortcuts")).toMatch(/Enter/);
    expect(rowOf(container, "a2").getAttribute("aria-keyshortcuts")).toBeNull();
    expect(rowOf(container, "p1").getAttribute("aria-keyshortcuts")).toMatch(/Enter/);
  });
});

describe("the count at the head of a group", () => {
  it("counts a task once when it shows under several bands", async () => {
    const all = await load();
    const both = byAgent("a1", { tags: ["x", "y"] });
    const { container } = show(all, [both, byAgent("a2", { tags: ["x"] }), task("p1")], {
      axis: "tag",
    });

    const agent = heads(container).find((one) => one.dataset.owner === "agent") as HTMLElement;

    expect(within(agent).getByText("2")).toBeTruthy();
  });
});
