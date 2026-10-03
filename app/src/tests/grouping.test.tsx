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
  task("1", "pay the power bill", "01H", ["home", "bills"]),
  task("2", "call the dentist", undefined, ["health"]),
  task("3", "read"),
];
const lists = [{ id: "01H", name: "Home", order: "a0" }];

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

    expect(heads()).toEqual(["Home1", "No list2"]);
  });

  it("puts a task under every one of its tags, and the untagged ones last", async () => {
    show();
    await groupBy(/Topic/);

    expect(heads()).toEqual(["#bills1", "#health1", "#home1", "Untagged1"]);
    expect(screen.getAllByText("pay the power bill")).toHaveLength(2);
  });

  it("folds a group away and opens it again", async () => {
    show();
    await groupBy(/List/);

    await userEvent.click(screen.getByRole("button", { name: /Home/ }));

    expect(screen.queryByText("pay the power bill")).toBeNull();
    expect(screen.getByText("read")).toBeTruthy();
  });

  it("remembers the grouping the next time the list is opened", async () => {
    const first = show();
    await groupBy(/List/);
    first.unmount();

    show();

    expect(heads()).toEqual(["Home1", "No list2"]);
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
      ...task("4", "renew the certificate", "01H"),
      resolved: { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1" },
    } as Task;
    show({ tasks: [...open, spoken] });
    await groupBy(/List/);

    expect(heads()).toEqual(["Home1", "No list2", "To confirm1"]);
  });

  it("opens a task an agent says should not be done instead of completing it", async () => {
    const select = vi.fn();
    const complete = vi.fn();
    const moot = {
      ...task("5", "migrate the reader"),
      resolved: { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1", drop: true },
    } as Task;
    show({ tasks: [moot], onSelect: select, onComplete: complete });

    await userEvent.click(screen.getByRole("button", { name: /should not be done/ }));

    expect(select).toHaveBeenCalledWith("5");
    expect(complete).not.toHaveBeenCalled();
  });
});

describe("walking a grouped list", () => {
  it("moves past a task's other copy when it is completed", async () => {
    localStorage.setItem("tisty.grouped", "tag");
    const two = [
      task("1", "pay the power bill", undefined, ["home", "bills"]),
      task("2", "read", undefined, ["health"]),
    ];
    render(
      <TaskList
        tasks={two}
        lists={[]}
        title="Open"
        bands="day"
        onSelect={() => {}}
        onComplete={() => {}}
      />,
    );
    const first = screen.getAllByRole("listitem")[0];
    first.focus();

    await userEvent.keyboard("{Control>}{Enter}{/Control}");

    expect(document.activeElement?.getAttribute("data-task")).toBe("2");
  });
});

describe("what a dropped task says of an agent's word", () => {
  it("says the agent had said it should not be done", () => {
    const gone = {
      ...task("6", "migrate the reader"),
      status: "dropped",
      resolved: { at: "2026-08-06T18:40:00Z", by: "dev_agent", entry: "e1", drop: true },
    } as Task;
    render(<TaskList tasks={[gone]} lists={[]} title="Archive" onSelect={() => {}} />);

    expect(screen.getByRole("listitem").getAttribute("aria-label")).toContain(
      "had said it should not be done",
    );
  });
});
