import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { fill, t } from "../locales";
import AgentWord, { agentWords, confirmWord } from "../ui/AgentWord";
import TaskList from "../ui/TaskList";
import { knowAgents } from "../who";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (said?: { drop?: boolean; via?: string }, more: Partial<Task> = {}): Task =>
  ({
    id: "1",
    title: "pasar biome",
    status: "open",
    priority: "unset",
    order: "a0",
    tags: [],
    reminders: [],
    resolved: said && { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1", ...said },
    ...more,
  }) as unknown as Task;

const REPEATS = { every: 1, unit: "week" } as unknown as Task["repeat"];

let listening = new AbortController();

afterEach(() => {
  listening.abort();
  listening = new AbortController();
  knowAgents({}, { clients: {} });
});

describe("the words an agent's claim carries", () => {
  it("says nothing for a task nobody spoke about", () => {
    expect(agentWords(task(), "agent")).toBeUndefined();
  });

  it.each([
    [false, "saidMarkDone"],
    [true, "saidMarkNotDoing"],
  ] as const)("is short when the signing is a mark (drop %s)", (drop, key) => {
    expect(agentWords(task({ drop, via: "claude" }), "mark")).toBe(t(key));
  });

  it.each([
    [false, "agentSaidDone"],
    [true, "agentSaidNotDoing"],
  ] as const)("is anonymous when the signing is an agent (drop %s)", (drop, key) => {
    expect(agentWords(task({ drop, via: "claude" }), "agent")).toBe(t(key));
  });

  it.each([
    [false, "agentNamedSaidDone"],
    [true, "agentNamedSaidNotDoing"],
  ] as const)("names the client when the signing is named (drop %s)", (drop, key) => {
    knowAgents({}, { clients: { claude: "Claude Code" } });

    expect(agentWords(task({ drop, via: "claude" }), "named")).toBe(fill(key, "Claude Code"));
  });

  it.each([
    [false, "agentSaidDone"],
    [true, "agentSaidNotDoing"],
  ] as const)("falls back to an agent when no client is known (drop %s)", (drop, key) => {
    expect(agentWords(task({ drop }), "named")).toBe(t(key));
  });
});

describe("the word that confirms", () => {
  it("completes what was said done", () => {
    expect(confirmWord(task({}))).toBe(t("markDone"));
  });

  it("discards what was said not to be done", () => {
    expect(confirmWord(task({ drop: true }))).toBe(t("discardIt"));
  });

  it("ends the repeat of a repeating task said not to be done", () => {
    const repeating = task({ drop: true }, { repeat: REPEATS });

    expect(confirmWord(repeating)).toBe(t("endRepeat"));
  });
});

describe("the card under a task an agent spoke about", () => {
  const show = (one: Task) => {
    const onConfirm = vi.fn();
    const onKeepOpen = vi.fn();
    const clicked = vi.fn();
    const { container } = render(
      <div>
        <AgentWord task={one} signing="agent" onConfirm={onConfirm} onKeepOpen={onKeepOpen} />
      </div>,
    );
    document.body.addEventListener("click", clicked, { signal: listening.signal });
    return { container, onConfirm, onKeepOpen, clicked };
  };

  it("renders nothing without a word from an agent", () => {
    const { container } = show(task());

    expect(container.querySelector("button")).toBeNull();
  });

  it.each(["done", "dropped"] as const)("renders nothing once the task is %s", (status) => {
    const { container } = show(task({}, { status }));

    expect(container.querySelector("button")).toBeNull();
  });

  it("shows the words and both buttons", () => {
    show(task({}));

    expect(screen.getByText(t("agentSaidDone"))).toBeTruthy();
    expect(screen.getByRole("button", { name: new RegExp(t("markDone")) })).toBeTruthy();
    expect(screen.getByRole("button", { name: new RegExp(t("stillOpen")) })).toBeTruthy();
  });

  it("confirms with the task, without clicking the row", async () => {
    const one = task({});
    const { onConfirm, onKeepOpen, clicked } = show(one);

    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("markDone")) }));

    expect(onConfirm).toHaveBeenCalledWith(one);
    expect(onKeepOpen).not.toHaveBeenCalled();
    expect(clicked).not.toHaveBeenCalled();
  });

  it("keeps the task open with the task, without clicking the row", async () => {
    const one = task({ drop: true });
    const { onConfirm, onKeepOpen, clicked } = show(one);

    await userEvent.click(screen.getByRole("button", { name: new RegExp(t("stillOpen")) }));

    expect(onKeepOpen).toHaveBeenCalledWith(one);
    expect(onConfirm).not.toHaveBeenCalled();
    expect(clicked).not.toHaveBeenCalled();
  });

  it("keeps keys pressed on its buttons from reaching the row", async () => {
    const reached = vi.fn();
    document.body.addEventListener("keydown", reached, { signal: listening.signal });
    show(task({}));

    screen.getByRole("button", { name: new RegExp(t("markDone")) }).focus();
    await userEvent.keyboard("{Enter}");
    screen.getByRole("button", { name: new RegExp(t("stillOpen")) }).focus();
    await userEvent.keyboard("x");

    expect(reached).not.toHaveBeenCalled();
  });

  it("marks the signing as a mark with a diamond and the full words on hover", () => {
    render(<AgentWord task={task({})} signing="mark" onConfirm={vi.fn()} onKeepOpen={vi.fn()} />);

    const words = screen.getByText(t("saidMarkDone"));

    expect(words.textContent).toBe(`◆ ${t("saidMarkDone")}`);
    expect(words.getAttribute("title")).toBe(t("agentSaidDone"));
  });

  it("explains why ending a repeat is not like the other discards", () => {
    const repeating = task({ drop: true }, { repeat: REPEATS });
    render(<AgentWord task={repeating} signing="agent" onConfirm={vi.fn()} onKeepOpen={vi.fn()} />);

    expect(
      screen.getByRole("button", { name: new RegExp(t("endRepeat")) }).getAttribute("title"),
    ).toBe(t("endRepeatWhy"));
  });
});

describe("the list of tasks", () => {
  const list = (props: { onConfirm?: () => void; onKeepOpen?: () => void }) =>
    render(
      <TaskList
        tasks={[task({})]}
        lists={[]}
        title="Open"
        confirming
        onSelect={() => {}}
        {...props}
      />,
    );

  it("shows the card when both answers are wired", () => {
    list({ onConfirm: vi.fn(), onKeepOpen: vi.fn() });

    expect(screen.getByText(t("agentSaidDone"))).toBeTruthy();
  });

  it.each([[{ onConfirm: vi.fn() }], [{ onKeepOpen: vi.fn() }], [{}]])(
    "keeps the card away unless both answers are wired",
    (props) => {
      list(props);

      expect(screen.queryByText(t("agentSaidDone"))).toBeNull();
    },
  );
});
