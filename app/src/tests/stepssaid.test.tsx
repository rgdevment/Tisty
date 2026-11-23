import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Card, Neuron } from "../brainsty";
import type { Task } from "../core";
import { fill, t } from "../locales";
import { factsOf } from "../ui/brainsty/cardFacts";
import Detail from "../ui/Detail";
import TaskList from "../ui/TaskList";

const parts: Task[] = [];
const asked: string[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) =>
    asked.push(cmd) &&
    Promise.resolve(
      cmd === "parts_of"
        ? parts
        : cmd === "task_story"
          ? { pages: [] }
          : cmd === "task_doc_open"
            ? null
            : [],
    ),
}));

const task = (id: string, title: string, more: Partial<Task> = {}): Task => ({
  id,
  title,
  status: "open",
  priority: "unset",
  order: id,
  tags: [],
  steps: [],
  log: [],
  reminders: [],
  ...more,
});

const waiting = fill("stepsWaiting", "1");

const listed = (tasks: Task[]) =>
  render(
    <TaskList tasks={tasks} lists={[]} title="Casa" onSelect={vi.fn()} onComplete={vi.fn()} />,
  );

describe("steps an agent said done, waiting on the person", () => {
  it("are counted apart in the row, beside the ticked ones", () => {
    listed([
      task("A", "renew the passport", {
        volume: { steps: 3, steps_done: 1, steps_said: 1, journal: 2 },
      }),
      task("B", "book the vet", { volume: { steps: 3, steps_done: 1, journal: 2 } }),
    ]);
    const [said, plain] = screen.getAllByRole("listitem");

    expect(said?.textContent).toContain("1/3◆1 · 1 to confirm · ✎2");
    expect(said?.querySelector(`[title="${waiting}"]`)).not.toBeNull();
    expect(plain?.textContent).toContain("1/3 · ✎2");
    expect(plain?.textContent).not.toContain("◆");
  });

  it("leave a row with no steps and no notes without a count", () => {
    listed([task("C", "call the plumber", { volume: {} })]);

    expect(screen.getByRole("listitem").textContent).not.toMatch(/\d\/\d|✎/);
  });

  it("are counted apart on a part's row too", async () => {
    parts.splice(0, parts.length, {
      ...task("P", "pack the books", { part_of: "W" }),
      volume: { steps: 2, steps_done: 0, steps_said: 1 },
    });
    render(
      <Detail
        task={task("W", "move house")}
        lists={[]}
        known={[]}
        onCollapse={vi.fn()}
        onPatch={vi.fn()}
        onComplete={vi.fn()}
        onDiscard={vi.fn()}
        onReopen={vi.fn()}
        onStillOpen={vi.fn()}
        onErase={vi.fn()}
        onFold={vi.fn()}
        onReadAs={vi.fn()}
        onClose={vi.fn()}
        whole={{ title: "move house", open: 1, closed: 0 }}
      />,
    );

    const shown = await screen.findByTitle(waiting);
    expect(shown.parentElement?.textContent).toBe("0/2◆1 · 1 to confirm");
  });

  it("are counted apart on Brainsty's card, the same as in the row", () => {
    const neuron: Neuron = { id: "A", kind: "task", title: "renew the passport", state: "open" };
    const card: Card = { id: "A", kind: "task", title: "renew the passport", state: "open" };
    const steps = (more: Partial<Card>) =>
      factsOf(neuron, { ...card, ...more }).find(
        ([label]) => label === t("brainstyFactSteps"),
      )?.[1];

    expect(steps({ steps: [1, 3], steps_said: 1 })).toBe(
      `${fill("brainstyFactStepsDone", "1", "3")} · ${waiting}`,
    );
    expect(steps({ steps: [1, 3] })).toBe(fill("brainstyFactStepsDone", "1", "3"));
    expect(steps({})).toBeUndefined();
  });
});

describe("answering an agent's word from the task", () => {
  const waiting = task("W", "renew the passport", {
    steps: [{ id: "s1", text: "Pay the fee", done: false, said: true, order: "a0" }],
  } as Partial<Task>);
  const shown = (onStepsSaid: (keep: boolean, steps: string[]) => Promise<unknown>) =>
    render(
      <Detail
        task={waiting}
        lists={[]}
        known={[]}
        onCollapse={vi.fn()}
        onPatch={vi.fn()}
        onComplete={vi.fn()}
        onDiscard={vi.fn()}
        onReopen={vi.fn()}
        onStillOpen={vi.fn()}
        onErase={vi.fn()}
        onFold={vi.fn()}
        onReadAs={vi.fn()}
        onClose={vi.fn()}
        onStepsSaid={onStepsSaid}
      />,
    );
  const opened = () => asked.filter((one) => one === "task_doc_open").length;
  const settled = () => new Promise((ready) => setTimeout(ready, 400));

  it("reads the page again once taking its word went through", async () => {
    asked.length = 0;
    const answered = vi.fn(() => Promise.resolve({}));
    shown(answered);
    await settled();
    const before = opened();

    screen.getByRole("button", { name: t("stepsSaidTake") }).click();
    await settled();

    expect(answered).toHaveBeenCalledWith(true, ["s1"]);
    await waitFor(() => expect(opened()).toBe(before + 1));
  });

  it("leaves the page as it is when taking its word fails, or when it is turned down", async () => {
    asked.length = 0;
    const failing = vi.fn(() => Promise.reject(new Error("refused")));
    const { unmount } = shown(failing);
    await settled();
    const before = opened();
    screen.getByRole("button", { name: t("stepsSaidTake") }).click();
    await settled();
    expect(opened()).toBe(before);
    unmount();

    asked.length = 0;
    const declined = vi.fn(() => Promise.resolve({}));
    shown(declined);
    await settled();
    const again = opened();
    screen.getByRole("button", { name: t("stepsSaidLeave") }).click();
    await settled();
    expect(declined).toHaveBeenCalledWith(false, ["s1"]);
    expect(opened()).toBe(again);
  });
});
