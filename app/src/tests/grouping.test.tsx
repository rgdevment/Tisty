import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Task } from "../core";
import TaskList from "../ui/TaskList";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const task = (id: string, title: string, list?: string, tags: string[] = []): Task =>
  ({
    id,
    title,
    status: "open",
    priority: "unset",
    order: "a0",
    list,
    tags,
    reminders: [],
  }) as unknown as Task;

const open = [
  task("1", "pagar la luz", "01H", ["casa", "pagos"]),
  task("2", "llamar al dentista", undefined, ["salud"]),
  task("3", "leer"),
];
const lists = [{ id: "01H", name: "Casa", order: "a0" }];

const show = (props: Partial<React.ComponentProps<typeof TaskList>> = {}) =>
  render(
    <TaskList tasks={open} lists={lists} title="Open" bands="day" onSelect={() => {}} {...props} />,
  );

const groupBy = async (word: RegExp) => {
  await userEvent.click(screen.getByRole("button", { name: "Grouped by" }));
  await userEvent.click(screen.getByRole("radio", { name: word }));
};

const heads = () =>
  screen
    .getAllByRole("button", { expanded: true })
    .map((one) => one.textContent?.replace(/^▾/, ""));

beforeEach(() => localStorage.clear());

describe("grouping the open tasks", () => {
  it("puts each task under its list, and the rest under no list", async () => {
    show();
    await groupBy(/List/);

    expect(heads()).toEqual(["Casa1", "No list2"]);
  });

  it("puts a task under every one of its tags, and the untagged ones last", async () => {
    show();
    await groupBy(/Topic/);

    expect(heads()).toEqual(["#casa1", "#pagos1", "#salud1", "Untagged1"]);
    expect(screen.getAllByText("pagar la luz")).toHaveLength(2);
  });

  it("folds a group away and opens it again", async () => {
    show();
    await groupBy(/List/);

    await userEvent.click(screen.getByRole("button", { name: /Casa/ }));

    expect(screen.queryByText("pagar la luz")).toBeNull();
    expect(screen.getByText("leer")).toBeTruthy();
  });

  it("remembers the grouping the next time the list is opened", async () => {
    const first = show();
    await groupBy(/List/);
    first.unmount();

    show();

    expect(heads()).toEqual(["Casa1", "No list2"]);
  });

  it("leaves the archive to its own grouping", () => {
    show({ bands: "month" });

    expect(screen.queryByRole("button", { name: "Grouped by" })).toBeNull();
  });

  it("is not offered where there are no day bands to trade, like a search or one list", () => {
    show({ bands: undefined });

    expect(screen.queryByRole("button", { name: "Grouped by" })).toBeNull();
  });

  it("keeps what an agent spoke for in its own band", async () => {
    const spoken = {
      ...task("4", "renovar el certificado", "01H"),
      resolved: { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1" },
    } as Task;
    show({ tasks: [...open, spoken] });
    await groupBy(/List/);

    expect(heads()).toEqual(["Casa1", "No list2", "To confirm1"]);
  });

  it("opens a task an agent says should not be done instead of completing it", async () => {
    const select = vi.fn();
    const complete = vi.fn();
    const moot = {
      ...task("5", "migrar el lector"),
      resolved: { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1", drop: true },
    } as Task;
    show({ tasks: [moot], onSelect: select, onComplete: complete });

    await userEvent.click(screen.getByRole("button", { name: /should not be done/ }));

    expect(select).toHaveBeenCalledWith("5");
    expect(complete).not.toHaveBeenCalled();
  });
});
