import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { List, Task } from "../core";
import { TASK } from "../drag";
import Sidebar from "../ui/Sidebar";
import TaskList from "../ui/TaskList";

const task = (id: string, title: string, over: Partial<Task> = {}): Task => ({
  id,
  title,
  status: "open",
  priority: 4,
  order: "a0",
  ...over,
});

const OPEN = [task("01A", "first"), task("01B", "second"), task("01C", "third")];

function held(id: string) {
  const data = new Map<string, string>([[TASK, id]]);
  return {
    dataTransfer: {
      types: [...data.keys()],
      getData: (kind: string) => data.get(kind) ?? "",
      setData: (kind: string, value: string) => void data.set(kind, value),
      effectAllowed: "move",
    },
  };
}

function list(tasks: Task[], onDrop?: (t: string, a?: string, b?: string) => void) {
  render(
    <TaskList
      tasks={tasks}
      lists={[]}
      title="Today"
      centred={false}
      onSelect={() => {}}
      onDrop={onDrop}
    />,
  );
}

const row = (title: string) => screen.getByText(title).closest("[draggable]");

describe("reordering by hand", () => {
  it("says where it landed by its neighbours, not by a number", () => {
    const onDrop = vi.fn();
    list(OPEN, onDrop);

    fireEvent.dragStart(row("third")!, held("01C"));
    fireEvent.drop(row("second")!, held("01C"));

    expect(onDrop).toHaveBeenCalledWith("01C", "01A", "01B");
  });

  it("dropping on the first row leaves nothing above it", () => {
    const onDrop = vi.fn();
    list(OPEN, onDrop);

    fireEvent.dragStart(row("third")!, held("01C"));
    fireEvent.drop(row("first")!, held("01C"));

    expect(onDrop).toHaveBeenCalledWith("01C", undefined, "01A");
  });

  it("dropping a task on itself does nothing", () => {
    const onDrop = vi.fn();
    list(OPEN, onDrop);

    fireEvent.dragStart(row("second")!, held("01B"));
    fireEvent.drop(row("second")!, held("01B"));

    expect(onDrop).not.toHaveBeenCalled();
  });

  it("is not offered where the order is not the user's to set", () => {
    list(OPEN);
    expect(screen.getByText("first").closest("[draggable]")).toBeNull();
  });
});

describe("what a drag is allowed to promise", () => {
  // The core sorts by date, then priority, then the manual key.
  const dated = [
    task("01A", "the report", { date: { at: "2026-08-10 09:00:00", tz: "", floating: false, has_time: false } }),
    task("01B", "the bank", { date: { at: "2026-08-11 09:00:00", tz: "", floating: false, has_time: false } }),
    task("01C", "the proxy", { date: { at: "2026-08-10 18:00:00", tz: "", floating: false, has_time: false } }),
  ];

  it("refuses a drop that the sort would undo", () => {
    const onDrop = vi.fn();
    render(
      <TaskList
        tasks={dated}
        lists={[]}
        title="Today"
        centred={false}
        onSelect={() => {}}
        onDrop={onDrop}
      />,
    );

    fireEvent.dragStart(row("the proxy")!, held("01C"));
    fireEvent.drop(row("the bank")!, held("01C"));

    expect(onDrop).not.toHaveBeenCalled();
  });

  it("allows it within the same day and priority", () => {
    const onDrop = vi.fn();
    render(
      <TaskList
        tasks={dated}
        lists={[]}
        title="Today"
        centred={false}
        onSelect={() => {}}
        onDrop={onDrop}
      />,
    );

    fireEvent.dragStart(row("the proxy")!, held("01C"));
    fireEvent.drop(row("the report")!, held("01C"));

    expect(onDrop).toHaveBeenCalledWith("01C", undefined, "01A");
  });
});

describe("filing by dropping on the sidebar", () => {
  const lists: List[] = [{ id: "01L", name: "work", order: "a0" }];

  it("sends the task to the list it was dropped on", () => {
    const onFile = vi.fn();
    render(
      <Sidebar
        lists={lists}
        counts={{ "01L": 2 }}
        chosen={{ named: "today" }}
        onChoose={() => {}}
        onFile={onFile}
      />,
    );

    fireEvent.drop(screen.getByRole("button", { name: /work/ }), held("01A"));
    expect(onFile).toHaveBeenCalledWith("01A", "01L");
  });

  it("the inbox takes it out of every list", () => {
    const onFile = vi.fn();
    render(
      <Sidebar
        lists={lists}
        counts={{}}
        chosen={{ named: "today" }}
        onChoose={() => {}}
        onFile={onFile}
      />,
    );

    fireEvent.drop(screen.getByRole("button", { name: /Inbox/ }), held("01A"));
    expect(onFile).toHaveBeenCalledWith("01A", undefined);
  });

  it("takes nothing when nobody offered a task", () => {
    const onFile = vi.fn();
    render(
      <Sidebar lists={lists} counts={{}} chosen={{ named: "today" }} onChoose={() => {}} onFile={onFile} />,
    );

    fireEvent.drop(screen.getByRole("button", { name: /work/ }), {
      dataTransfer: { types: [], getData: () => "" },
    });
    expect(onFile).not.toHaveBeenCalled();
  });
});
