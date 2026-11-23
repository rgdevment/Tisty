import { invoke } from "@tauri-apps/api/core";
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import { fill, t } from "../locales";
import Detail from "../ui/Detail";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve(null)) }));

beforeEach(() => {
  vi.mocked(invoke).mockImplementation((cmd: string) => {
    if (cmd === "task_story") {
      return Promise.resolve({
        id: "01A",
        pages: [{ n: 1, at: "2026-08-01 09:00:00", chapter: "made" }],
      });
    }
    if (cmd === "task_refs") return Promise.resolve([]);
    return Promise.resolve(null);
  });
});

const one = (status: Task["status"]): Task =>
  ({
    id: "01A",
    title: "write the report",
    status,
    priority: "unset",
    order: "a0",
    description: "the one for accounting",
    steps: [],
    log: [],
    volume: {},
  }) as unknown as Task;

const show = (status: Task["status"]) =>
  render(
    <Detail
      task={one(status)}
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

describe("the side block with the trail", () => {
  it("keeps one of each full-screen too, for a settled task", async () => {
    show("done");
    await waitFor(() => expect(screen.getAllByText("The trail").length).toBeGreaterThan(0));

    expect(screen.getAllByText("The trail")).toHaveLength(1);
    expect(screen.getAllByText("What it carries")).toHaveLength(1);
  });

  it("keeps the sealed notice for a settled task, in the column and full-screen", async () => {
    const said = /This trail is not edited/;
    show("dropped");
    await waitFor(() => expect(screen.getAllByText(said)).toHaveLength(1));
  });

  it("does not put the sealed notice over an open task full-screen", async () => {
    show("open");
    await waitFor(() => expect(screen.getAllByText("The trail").length).toBeGreaterThan(0));

    expect(screen.queryAllByText(/This trail is not edited/)).toHaveLength(0);
  });
});

describe("what a task carries", () => {
  it("counts the steps an agent said done apart, as the row does", async () => {
    const said = { ...one("open"), volume: { steps: 3, steps_done: 1, steps_said: 1 } };
    render(
      <Detail
        task={said}
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

    const count = await screen.findByText(fill("stepsWaiting", "1"), { exact: false });
    expect(count.closest("dt")?.textContent).toBe(`1/3◆1 · ${fill("stepsWaiting", "1")}`);
  });

  it("still says 0/0 for a task with no steps", async () => {
    show("open");
    const steps = await screen.findByText(t("factSteps"));
    expect(steps.previousElementSibling?.textContent).toBe("0/0");
  });
});
