import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task, Whole } from "../core";
import { fill, t } from "../locales";
import Detail from "../ui/Detail";
import Steps from "../ui/Steps";
import TaskList from "../ui/TaskList";

const parts: Task[] = [];
const offered: { id: string; title: string }[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) =>
    Promise.resolve(cmd === "parts_of" ? parts : cmd === "wholes_offered" ? offered : []),
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

beforeEach(() => {
  parts.splice(0, parts.length);
  offered.splice(0, offered.length);
});

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

    fireEvent.click(screen.getByRole("button", { name: "⌂ move house" }));

    expect(back).toHaveBeenCalledWith("W");
  });
});

describe("parts in plain sight", () => {
  it("offers a part to a task that has none, under its steps", () => {
    parts.splice(0, parts.length);
    opened(task("T", "test the login"), { onAddPart: vi.fn() });

    expect(screen.getByText(t("parts"))).not.toBeNull();
    expect(screen.getByLabelText(t("addPart"))).not.toBeNull();
  });

  it("offers no parts to a task that comes back", () => {
    opened(
      task("T", "water the plants", {
        repeat: { from: "due", each: { every: 1, unit: "week" } },
      }),
      { onAddPart: vi.fn() },
    );
    expect(screen.queryByLabelText(t("addPart"))).toBeNull();
  });

  it("lets a part go of its whole from where the whole is named", () => {
    const hang = vi.fn();
    opened(task("P", "pack", { part_of: "W" }), { partOf: "move house", onHang: hang });

    fireEvent.click(screen.getByRole("button", { name: fill("letGoOf", "move house") }));

    expect(hang).toHaveBeenCalledWith(null);
  });

  it("gives a part no line to add parts of its own", () => {
    opened(task("P", "pack", { part_of: "W" }), { partOf: "move house", onAddPart: vi.fn() });

    expect(screen.queryByLabelText(t("addPart"))).toBeNull();
  });

  it("still lets go of a whole whose name has not come in yet", () => {
    const hang = vi.fn();
    opened(task("P", "pack", { part_of: "W" }), { onHang: hang });

    fireEvent.click(screen.getByRole("button", { name: fill("letGoOf", "…") }));

    expect(hang).toHaveBeenCalledWith(null);
  });

  it("finds a whole whatever its accents", async () => {
    offered.splice(0, offered.length, { id: "R", title: "Reunión con el banco" });
    const hang = vi.fn();
    opened(task("T", "pack the books"), { onAddPart: vi.fn(), onHang: hang });

    fireEvent.click(await screen.findByRole("button", { name: `⌂ ${t("partOfOther")}` }));
    fireEvent.change(screen.getByLabelText(t("partOfFind")), {
      target: { value: "banco reunion" },
    });

    expect(screen.getByRole("button", { name: "Reunión con el banco" })).not.toBeNull();
  });

  it("does not hang a task on Enter before anything is typed", async () => {
    offered.splice(0, offered.length, { id: "M", title: "move house" });
    const hang = vi.fn();
    opened(task("T", "pack the books"), { onAddPart: vi.fn(), onHang: hang });

    fireEvent.click(await screen.findByRole("button", { name: `⌂ ${t("partOfOther")}` }));
    fireEvent.keyDown(screen.getByLabelText(t("partOfFind")), { key: "Enter" });

    expect(hang).not.toHaveBeenCalled();
  });

  it("keeps the finder out of the line that adds a part, so Enter there still adds", async () => {
    offered.splice(0, offered.length, { id: "M", title: "move house" });
    opened(task("T", "pack the books"), { onAddPart: vi.fn(), onHang: vi.fn() });

    fireEvent.click(await screen.findByRole("button", { name: `⌂ ${t("partOfOther")}` }));

    expect(screen.getByLabelText(t("partOfFind")).closest("form")).toBeNull();
  });

  it("does not ask for wholes a part could never hang from", () => {
    opened(task("P", "pack", { part_of: "W" }), { partOf: "move house", onHang: vi.fn() });

    expect(screen.queryByRole("button", { name: `⌂ ${t("partOfOther")}` })).toBeNull();
  });

  it("finds the whole by name, a handful at a time", async () => {
    parts.splice(0, parts.length);
    offered.splice(
      0,
      offered.length,
      ...Array.from({ length: 8 }, (_, n) => ({ id: `W${n}`, title: `release ${n}` })),
      { id: "M", title: "move house" },
    );
    const hang = vi.fn();
    opened(task("T", "pack the books"), { onAddPart: vi.fn(), onHang: hang });

    fireEvent.click(await screen.findByRole("button", { name: `⌂ ${t("partOfOther")}` }));
    expect(screen.getByText(fill("partOfMore", "4"))).not.toBeNull();

    fireEvent.change(screen.getByLabelText(t("partOfFind")), { target: { value: "move" } });
    fireEvent.click(screen.getByRole("button", { name: "move house" }));

    expect(hang).toHaveBeenCalledWith("M");
  });

  it("says so when no open task goes by that name", async () => {
    offered.splice(0, offered.length, { id: "M", title: "move house" });
    opened(task("T", "pack the books"), { onAddPart: vi.fn(), onHang: vi.fn() });

    fireEvent.click(await screen.findByRole("button", { name: `⌂ ${t("partOfOther")}` }));
    fireEvent.change(screen.getByLabelText(t("partOfFind")), { target: { value: "garden" } });

    expect(screen.getByText(t("partOfNone"))).not.toBeNull();
  });

  it("does not offer a whole to a task that holds parts of its own", async () => {
    parts.splice(0, parts.length, task("P", "pack", { part_of: "W" }));
    offered.splice(0, offered.length, { id: "M", title: "move house" });
    opened(task("W", "move house"), { whole, onAddPart: vi.fn(), onHang: vi.fn() });

    await waitFor(() => expect(screen.getByText("pack")).not.toBeNull());
    expect(screen.queryByRole("button", { name: `⌂ ${t("partOfOther")}` })).toBeNull();
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

describe("what the review asked of parts", () => {
  it("leaves the parts as rows in a dense listing, which nests nothing", () => {
    render(
      <TaskList
        tasks={[task("W", "move house"), task("P", "pack", { part_of: "W" })]}
        lists={[]}
        wholes={{ W: whole }}
        title="Archivo"
        dense
        onSelect={vi.fn()}
      />,
    );

    expect(screen.getByText("pack")).not.toBeNull();
  });

  it("keeps the count on a whole an assistant said done", () => {
    listed([
      task("W", "move house", { resolved: { at: "2026-10-05T10:00:00Z" } as Task["resolved"] }),
      task("P", "pack", { part_of: "W" }),
    ]);

    expect(screen.getByText(/▣ 1\/2/)).not.toBeNull();
  });

  it("counts what is out of sight among what closing lets go", () => {
    parts.splice(0, parts.length);
    opened(task("W", "move house"), {
      whole: { title: "move house", open: 0, closed: 1, away: 2 },
      onAddPart: vi.fn(),
    });

    expect(screen.getByText(fill("partsLeft", "2"))).not.toBeNull();
  });
});

describe("moving through a whole's parts with the keyboard", () => {
  it("keeps the tab stop on the part last reached", () => {
    listed([task("W", "move house"), task("P", "pack", { part_of: "W" })]);
    const part = screen.getAllByRole("listitem")[1] as HTMLElement;

    fireEvent.focus(part);

    expect(part.tabIndex).toBe(0);
  });
});
