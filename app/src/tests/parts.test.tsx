import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Task, Whole } from "../core";
import { fill, t } from "../locales";
import Detail from "../ui/Detail";
import Steps from "../ui/Steps";
import TaskList from "../ui/TaskList";

const parts: Task[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "parts_of" ? parts : []),
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

const whole: Whole = { title: "move house", open: 1, closed: 1 };

const listed = (tasks: Task[], wholes: Record<string, Whole> = { W: whole }) =>
  render(
    <TaskList
      tasks={tasks}
      lists={[]}
      wholes={wholes}
      title="Casa"
      onSelect={vi.fn()}
      onComplete={vi.fn()}
    />,
  );

const opened = (shown: Task, more: Partial<Parameters<typeof Detail>[0]> = {}) =>
  render(
    <Detail
      task={shown}
      lists={[]}
      known={[]}
      expanded={false}
      onExpand={vi.fn()}
      onCollapse={vi.fn()}
      onPatch={vi.fn()}
      onStep={vi.fn()}
      onMark={vi.fn()}
      onDropStep={vi.fn()}
      onLog={vi.fn()}
      onComplete={vi.fn()}
      onDiscard={vi.fn()}
      onReopen={vi.fn()}
      onStillOpen={vi.fn()}
      onErase={vi.fn()}
      onFold={vi.fn()}
      onReadAs={vi.fn()}
      onOpenToAgents={vi.fn()}
      onClose={vi.fn()}
      {...more}
    />,
  );

describe("a task with parts in a list", () => {
  it("holds its parts beneath it instead of beside it", () => {
    listed([task("W", "move house"), task("P", "pack", { part_of: "W" })]);

    const rows = screen.getAllByRole("listitem");
    expect(rows).toHaveLength(2);
    expect(rows[1]?.closest(".border-l")).not.toBeNull();
  });

  it("counts its parts and folds them away from the count", () => {
    listed([task("W", "move house"), task("P", "pack", { part_of: "W" })]);
    const said = fill("partsClosed", "1", "2");
    const count = screen.getByRole("button", { name: new RegExp(said) });

    expect(count.textContent).toContain("▣ 1/2");
    expect(count.getAttribute("aria-expanded")).toBe("true");
    fireEvent.click(count);

    expect(screen.getAllByRole("listitem")).toHaveLength(1);
  });

  it("names its whole when the whole is not in the view", () => {
    listed([task("P", "pack", { part_of: "W" })]);

    expect(screen.getByTitle(fill("partOf", "move house")).textContent).toContain("⌂ move house");
  });
});

describe("the detail of a task with parts", () => {
  it("lists its parts and says how many are left before closing them", async () => {
    parts.splice(0, parts.length, task("P", "pack", { part_of: "W" }));
    opened(task("W", "move house"), { whole, onAddPart: vi.fn() });

    await waitFor(() => expect(screen.getByText("pack")).not.toBeNull());
    expect(screen.getByText(t("parts"))).not.toBeNull();
    expect(screen.getByText(fill("partsLeft", "1"))).not.toBeNull();
    expect(screen.getByRole("button", { name: new RegExp(t("closeLettingGo")) })).not.toBeNull();
  });

  it("adds a part from the line beneath them", async () => {
    parts.splice(0, parts.length);
    const added = vi.fn();
    opened(task("W", "move house"), { whole, onAddPart: added });

    fireEvent.change(screen.getByLabelText(t("addPart")), { target: { value: "call the bank" } });
    fireEvent.submit(screen.getByLabelText(t("addPart")).closest("form") as HTMLFormElement);

    expect(added).toHaveBeenCalledWith("call the bank");
  });

  it("closes as usual once nothing is left open", () => {
    parts.splice(0, parts.length);
    opened(task("W", "move house"), { whole: { ...whole, open: 0 }, onAddPart: vi.fn() });

    expect(screen.queryByText(t("closeLettingGo"))).toBeNull();
    expect(screen.getByRole("button", { name: new RegExp(t("markDone")) })).not.toBeNull();
  });

  it("lets a part lead back to its whole", () => {
    const back = vi.fn();
    opened(task("P", "pack", { part_of: "W" }), { partOf: "move house", onOpenPart: back });

    fireEvent.click(screen.getByText(`⌂ ${fill("partOf", "move house")}`));

    expect(back).toHaveBeenCalledWith("W");
  });
});

describe("a step that outgrows itself", () => {
  it("is turned into a task in one gesture", () => {
    const turned = vi.fn();
    render(
      <Steps
        steps={[{ id: "s1", text: "call the landlord", done: false, order: "a0" }]}
        onWrite={vi.fn()}
        onMark={vi.fn()}
        onDrop={vi.fn()}
        onTurn={turned}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: `${t("turnIntoTask")}: call the landlord` }),
    );

    expect(turned).toHaveBeenCalledWith("s1");
  });

  it("is not offered for a step already done", () => {
    render(
      <Steps
        steps={[{ id: "s1", text: "call the landlord", done: true, order: "a0" }]}
        onWrite={vi.fn()}
        onMark={vi.fn()}
        onDrop={vi.fn()}
        onTurn={vi.fn()}
      />,
    );

    expect(screen.queryByRole("button", { name: new RegExp(t("turnIntoTask")) })).toBeNull();
  });
});
